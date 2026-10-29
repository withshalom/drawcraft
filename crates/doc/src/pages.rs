//! Pages (draw mode): a page order and master layers over the artboards.
//!
//! A page is an artboard. A document becomes a pages document when it gets a [`Pages`] table
//! ([`Document::pages`]); without one it is an ordinary artboards document and saves as before.
//! The table keys on ids the document already has (artboard ids, top-level layer ids), so every
//! artboard command and every exporter keeps working on a pages document unchanged:
//!
//! - **Page order** ([`Document::page_order`]): the artboards in the order the table lists them,
//!   then any it doesn't list (new ones), in document order. Ids of deleted artboards are skipped,
//!   so deleting an artboard never leaves the table wrong.
//! - **Master layers** ([`Document::master_layer_ids`]): top-level layers whose art is drawn on
//!   every page. Their art is laid out on the first page; on each other page it repeats at the same
//!   place relative to that page's top-left corner. Output that goes through
//!   [`Document::with_master_pages`] (every exporter, print and the canvas) draws the repeats; the
//!   document itself holds the art once.
//!
//! A VectorCraft without pages reads such a file as its artboards and keeps the `pages` table it
//! doesn't know (`Document::extra`), so it survives a round trip there.

use std::borrow::Cow;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use vectorcraft_geom::{Affine, Vec2};

use crate::{Document, Node, NodeId};

/// Most pages a document's table lists, and most master layers; longer lists are cut as files
/// open (untrusted input) and the commands refuse to grow past them.
pub const MAX_PAGES: usize = 10_000;
/// Most master layers.
pub const MAX_MASTER_LAYERS: usize = 64;
/// Most objects [`Document::with_master_pages`] adds as repeats: past it, the remaining pages show
/// no master art rather than make output unboundedly large.
pub const MAX_MASTER_COPIES: usize = 1_000_000;

/// A document's page table ([`Document::pages`]).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pages {
    /// Artboard ids ([`crate::Artboard::id`]) in page order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<u32>,
    /// Ids of the top-level layers that are master layers, in no particular order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub master_layers: Vec<NodeId>,
}

impl Pages {
    /// The table with lists cut to [`MAX_PAGES`] and [`MAX_MASTER_LAYERS`] and repeated ids
    /// dropped (a file is untrusted).
    pub fn bounded(mut self) -> Self {
        let mut seen = std::collections::HashSet::new();
        self.order.retain(|id| seen.insert(*id));
        self.order.truncate(MAX_PAGES);
        let mut seen = std::collections::HashSet::new();
        self.master_layers.retain(|id| seen.insert(*id));
        self.master_layers.truncate(MAX_MASTER_LAYERS);
        self
    }
}

impl Document {
    /// Whether this is a pages document (it has a [`Pages`] table).
    pub fn is_paged(&self) -> bool {
        self.pages.is_some()
    }

    /// Indices into [`Document::artboards`] in page order: those the table lists, then the rest in
    /// document order. Without a table, document order.
    pub fn page_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = Vec::with_capacity(self.artboards.len());
        let mut used = vec![false; self.artboards.len()];
        if let Some(p) = &self.pages {
            // Artboard ids are unique; should a damaged file repeat one, its first artboard counts.
            let mut at = std::collections::HashMap::with_capacity(self.artboards.len());
            for (i, a) in self.artboards.iter().enumerate() {
                at.entry(a.id).or_insert(i);
            }
            for id in &p.order {
                if let Some(&i) = at.get(id)
                    && let Some(u) = used.get_mut(i)
                    && !*u
                {
                    *u = true;
                    order.push(i);
                }
            }
        }
        order.extend((0..self.artboards.len()).filter(|i| !used.get(*i).copied().unwrap_or(true)));
        order
    }

    /// The master layers that exist: top-level layers the table lists, bottom first.
    pub fn master_layer_ids(&self) -> Vec<NodeId> {
        let Some(p) = &self.pages else { return vec![] };
        self.layers.iter().filter(|l| l.is_layer() && p.master_layers.contains(&l.id)).map(|l| l.id).collect()
    }

    /// Whether `id` is a master layer.
    pub fn is_master_layer(&self, id: NodeId) -> bool {
        self.pages.as_ref().is_some_and(|p| p.master_layers.contains(&id)) && self.layers.iter().any(|l| l.id == id && l.is_layer())
    }

    /// The offset of each page after the first from the first page (top-left corner to top-left
    /// corner), in page order: where master art repeats.
    pub fn master_offsets(&self) -> Vec<Vec2> {
        let order = self.page_order();
        let Some(first) = order.first().and_then(|i| self.artboards.get(*i)) else { return vec![] };
        let origin = first.rect.abs().origin();
        order
            .iter()
            .skip(1)
            .filter_map(|i| self.artboards.get(*i))
            .map(|a| a.rect.abs().origin() - origin)
            .filter(|v| v.x.is_finite() && v.y.is_finite())
            .collect()
    }

    /// This document as output draws it: each master layer holding, after its own art, a copy of
    /// that art moved onto every other page ([`Document::master_offsets`]), with ids of their own.
    /// The document itself when there is nothing to repeat (no table, no master layer with art, or
    /// one page), so ordinary documents cost nothing. The copy lists no master layers, so applying
    /// this again adds nothing.
    pub fn with_master_pages(&self) -> Cow<'_, Document> {
        let masters = self.master_layer_ids();
        let offsets = self.master_offsets();
        if masters.is_empty() || offsets.is_empty() {
            return Cow::Borrowed(self);
        }
        let has_art = self.layers.iter().any(|l| masters.contains(&l.id) && l.children().is_some_and(|c| !c.is_empty()));
        if !has_art {
            return Cow::Borrowed(self);
        }
        let mut d = self.clone();
        let mut budget = MAX_MASTER_COPIES;
        let layers = std::mem::take(&mut d.layers);
        let mut out = Vec::with_capacity(layers.len());
        for layer in layers {
            if !masters.contains(&layer.id) {
                out.push(layer);
                continue;
            }
            let mut l = (*layer).clone();
            let originals: Vec<Arc<Node>> = l.children().cloned().unwrap_or_default();
            let per_page = originals.iter().map(|n| count(n)).sum::<usize>().max(1);
            let mut copies = Vec::new();
            for off in &offsets {
                if budget < per_page {
                    break;
                }
                budget -= per_page;
                for n in &originals {
                    let mut c = (**n).clone();
                    c.transform(Affine::translate(*off), false);
                    renumber(&mut d, &mut c);
                    copies.push(Arc::new(c));
                }
            }
            if let Some(ch) = l.children_mut() {
                ch.extend(copies);
            }
            out.push(Arc::new(l));
        }
        d.layers = out;
        // The repeats are made: a second pass (an exporter after the encoder) adds none.
        if let Some(p) = &mut d.pages {
            p.master_layers.clear();
        }
        d.drop_lookup_cache();
        Cow::Owned(d)
    }
}

/// Objects in `n`'s tree, itself included.
fn count(n: &Node) -> usize {
    1 + n.children().map_or(0, |c| c.iter().map(|n| count(n)).sum())
}

/// Give `n` and everything in it new ids from `d`.
fn renumber(d: &mut Document, n: &mut Node) {
    n.id = d.alloc_id();
    for c in n.children_mut().into_iter().flatten() {
        renumber(d, Arc::make_mut(c));
    }
}

#[cfg(test)]
mod tests;
