use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Rect {
    fn new_empty() -> Self {
        Rect {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    fn from_js(item: &JsValue) -> Self {
        fn get_coord(val: &JsValue, prop: &str) -> f64 {
            match Reflect::get(val, &JsValue::from_str(prop)) {
                Ok(v) => v.as_f64().unwrap_or(0.0),
                Err(_) => 0.0,
            }
        }
        Rect {
            min_x: get_coord(item, "minX"),
            min_y: get_coord(item, "minY"),
            max_x: get_coord(item, "maxX"),
            max_y: get_coord(item, "maxY"),
        }
    }

    fn from_flat(data: &[f64]) -> Self {
        Rect {
            min_x: data[0],
            min_y: data[1],
            max_x: data[2],
            max_y: data[3],
        }
    }

    fn area(&self) -> f64 {
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }

    fn margin(&self) -> f64 {
        (self.max_x - self.min_x) + (self.max_y - self.min_y)
    }

    fn enlarged_area(&self, other: &Rect) -> f64 {
        (f64::max(other.max_x, self.max_x) - f64::min(other.min_x, self.min_x))
            * (f64::max(other.max_y, self.max_y) - f64::min(other.min_y, self.min_y))
    }

    fn intersection_area(&self, other: &Rect) -> f64 {
        let min_x = f64::max(self.min_x, other.min_x);
        let min_y = f64::max(self.min_y, other.min_y);
        let max_x = f64::min(self.max_x, other.max_x);
        let max_y = f64::min(self.max_y, other.max_y);

        f64::max(0.0, max_x - min_x) * f64::max(0.0, max_y - min_y)
    }

    fn contains(&self, other: &Rect) -> bool {
        self.min_x <= other.min_x
            && self.min_y <= other.min_y
            && other.max_x <= self.max_x
            && other.max_y <= self.max_y
    }

    fn intersects(&self, other: &Rect) -> bool {
        other.min_x <= self.max_x
            && other.min_y <= self.max_y
            && other.max_x >= self.min_x
            && other.max_y >= self.min_y
    }

    fn extend(&mut self, other: &Rect) {
        self.min_x = f64::min(self.min_x, other.min_x);
        self.min_y = f64::min(self.min_y, other.min_y);
        self.max_x = f64::max(self.max_x, other.max_x);
        self.max_y = f64::max(self.max_y, other.max_y);
    }
}

#[derive(Clone)]
struct Entry {
    bbox: Rect,
    data: JsValue,
    is_leaf: bool,
    height: usize,
    children: Vec<Entry>,
}

impl Entry {
    fn new_leaf(item: JsValue) -> Self {
        let bbox = Rect::from_js(&item);
        Entry {
            bbox,
            data: item,
            is_leaf: true,
            height: 1,
            children: vec![],
        }
    }

    fn new_node(children: Vec<Entry>) -> Self {
        let mut node = Entry {
            bbox: Rect::new_empty(),
            data: JsValue::NULL,
            is_leaf: false,
            height: 1,
            children,
        };
        node.calc_bbox();
        node
    }

    fn calc_bbox(&mut self) {
        let mut dist_bbox = Rect::new_empty();
        for child in &self.children {
            dist_bbox.extend(&child.bbox);
        }
        self.bbox = dist_bbox;
    }
}

#[wasm_bindgen]
pub struct RBush {
    root: Entry,
    max_entries: usize,
    min_entries: usize,
}

#[wasm_bindgen]
impl RBush {
    #[wasm_bindgen(constructor)]
    pub fn new(max_entries: Option<usize>) -> RBush {
        let m = max_entries.unwrap_or(9).max(4);
        let min = (m as f64 * 0.4).ceil().max(2.0) as usize;

        RBush {
            root: Entry::new_node(vec![]),
            max_entries: m,
            min_entries: min,
        }
    }

    pub fn clear(&mut self) {
        self.root = Entry::new_node(vec![]);
    }

    pub fn all(&self) -> Array {
        let result = Array::new();
        self._all(&self.root, &result);
        result
    }

    pub fn search(&self, bbox_js: &JsValue) -> Array {
        let bbox = Rect::from_js(bbox_js);
        let result = Array::new();
        let mut stack = vec![&self.root];

        while let Some(node) = stack.pop() {
            if !bbox.intersects(&node.bbox) {
                continue;
            }

            for child in &node.children {
                if bbox.intersects(&child.bbox) {
                    if child.is_leaf {
                        result.push(&child.data);
                    } else if bbox.contains(&child.bbox) {
                        self._all(child, &result);
                    } else {
                        stack.push(child);
                    }
                }
            }
        }
        result
    }

    pub fn collides(&self, bbox_js: &JsValue) -> bool {
        let bbox = Rect::from_js(bbox_js);
        let mut stack = vec![&self.root];

        while let Some(node) = stack.pop() {
            if !bbox.intersects(&node.bbox) {
                continue;
            }

            for child in &node.children {
                if bbox.intersects(&child.bbox) {
                    if child.is_leaf || bbox.contains(&child.bbox) {
                        return true;
                    }
                    stack.push(child);
                }
            }
        }
        false
    }

    #[wasm_bindgen(js_name = insert)]
    pub fn insert(&mut self, item: JsValue) {
        let entry = Entry::new_leaf(item);
        self.insert_entry(entry);
    }

    pub fn load(&mut self, data: &Array) {
        if data.length() == 0 {
            return;
        }

        let items: Vec<Entry> = (0..data.length())
            .map(|i| Entry::new_leaf(data.get(i)))
            .collect();

        self.bulk_load(items);
    }

    #[wasm_bindgen(js_name = loadHybrid)]
    pub fn load_hybrid(&mut self, fast_coords: &[f64], items: &Array) {
        if fast_coords.is_empty() {
            return;
        }

        let count = fast_coords.len() / 4;
        let mut entries = Vec::with_capacity(count);

        for i in 0..count {
            let start = i * 4;
            let bbox = Rect::from_flat(&fast_coords[start..start + 4]);
            let item_data = items.get(i as u32);

            entries.push(Entry {
                bbox,
                data: item_data,
                is_leaf: true,
                height: 1,
                children: vec![],
            });
        }

        self.bulk_load(entries);
    }

    pub fn remove(&mut self, item: JsValue) {
        let bbox = Rect::from_js(&item);
        let mut items_to_reinsert = Vec::new();

        RBush::remove_from_node(
            &mut self.root,
            &item,
            &bbox,
            self.min_entries,
            &mut items_to_reinsert,
        );

        for item in items_to_reinsert {
            self.insert_entry(item);
        }

        if !self.root.is_leaf && self.root.children.len() == 1 {
            self.root = self.root.children.pop().unwrap();
        }
    }

    fn remove_from_node(
        node: &mut Entry,
        item: &JsValue,
        bbox: &Rect,
        min_entries: usize,
        reinsert: &mut Vec<Entry>,
    ) -> bool {
        if node.is_leaf {
            let mut index = None;
            for (i, child) in node.children.iter().enumerate() {
                if &child.data == item {
                    index = Some(i);
                    break;
                }
            }

            if let Some(idx) = index {
                node.children.remove(idx);
                node.calc_bbox();
                return true;
            }
            return false;
        }

        let mut removed = false;
        let mut removal_index = None;

        for (i, child) in node.children.iter_mut().enumerate() {
            if child.bbox.contains(bbox) {
                if RBush::remove_from_node(child, item, bbox, min_entries, reinsert) {
                    removed = true;
                    if child.children.len() < min_entries {
                        removal_index = Some(i);
                    } else {
                        child.calc_bbox();
                    }
                    break;
                }
            }
        }

        if let Some(idx) = removal_index {
            let underflowed_child = node.children.remove(idx);
            RBush::collect_items(&underflowed_child, reinsert);
            node.calc_bbox();
        } else if removed {
            node.calc_bbox();
        }

        removed
    }

    fn collect_items(node: &Entry, acc: &mut Vec<Entry>) {
        if node.is_leaf {
            for child in &node.children {
                acc.push(child.clone());
            }
        } else {
            for child in &node.children {
                RBush::collect_items(child, acc);
            }
        }
    }

    fn bulk_load(&mut self, mut items: Vec<Entry>) {
        if items.len() < self.min_entries {
            for item in items {
                self.insert_entry(item);
            }
            return;
        }

        let len = items.len();
        let node = self._build(&mut items, 0, len - 1, 0);

        if self.root.children.is_empty() {
            self.root = node;
        } else if self.root.height == node.height {
            self._split_root(node);
        } else {
            if self.root.height < node.height {
                let tmp = self.root.clone();
                self.root = node;
                let level = self.root.height - tmp.height - 1;
                self._insert_at_level(tmp, level);
            } else {
                let level = self.root.height - node.height - 1;
                self._insert_at_level(node, level);
            }
        }
    }

    fn insert_entry(&mut self, item: Entry) {
        let level = self.root.height - 1;
        self._insert_at_level(item, level);
    }

    fn _insert_at_level(&mut self, item: Entry, level: usize) {
        let split = RBush::insert_recursive(
            &mut self.root,
            item,
            level,
            self.max_entries,
            self.min_entries,
        );
        if let Some(new_node) = split {
            self._split_root(new_node);
        }
    }

    fn _split_root(&mut self, new_node: Entry) {
        let old_root_children = std::mem::take(&mut self.root.children);
        let mut old_root = Entry::new_node(old_root_children);
        old_root.height = self.root.height;
        old_root.calc_bbox();

        self.root.height += 1;
        self.root.is_leaf = false;
        self.root.children = vec![old_root, new_node];
        self.root.calc_bbox();
    }

    fn _all(&self, node: &Entry, result: &Array) {
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            for child in &n.children {
                if child.is_leaf {
                    result.push(&child.data);
                } else {
                    stack.push(child);
                }
            }
        }
    }

    fn _build(&self, items: &mut [Entry], left: usize, right: usize, height: usize) -> Entry {
        let n = right - left + 1;
        let mut m = self.max_entries;

        if n <= m {
            let children = items[left..=right].to_vec();
            return Entry::new_node(children);
        }

        let mut target_height = height;
        if target_height == 0 {
            target_height = (n as f64).log(m as f64).ceil() as usize;
            m = (n as f64 / (m as f64).powi((target_height - 1) as i32)).ceil() as usize;
        }

        let mut node = Entry::new_node(vec![]);
        node.height = target_height;

        let n2 = (n as f64 / m as f64).ceil() as usize;
        let n1 = n2 * (m as f64).sqrt().ceil() as usize;

        RBush::multi_select(items, left, right, n1, true);

        let mut children = vec![];
        let mut i = left;
        while i <= right {
            let right2 = std::cmp::min(i + n1 - 1, right);

            RBush::multi_select(items, i, right2, n2, false);

            let mut j = i;
            while j <= right2 {
                let right3 = std::cmp::min(j + n2 - 1, right2);
                children.push(self._build(items, j, right3, target_height - 1));
                j += n2;
            }
            i += n1;
        }

        node.children = children;
        node.calc_bbox();
        node
    }

    fn multi_select(arr: &mut [Entry], left: usize, right: usize, n: usize, compare_x: bool) {
        let mut stack = vec![(left, right)];

        while let Some((l, r)) = stack.pop() {
            if r - l <= n {
                continue;
            }

            let mid = l + ((r - l) as f64 / n as f64 / 2.0).ceil() as usize * n;
            let target_idx = mid - l;
            let slice = &mut arr[l..=r];

            if compare_x {
                slice.select_nth_unstable_by(target_idx, |a, b| {
                    a.bbox.min_x.partial_cmp(&b.bbox.min_x).unwrap()
                });
            } else {
                slice.select_nth_unstable_by(target_idx, |a, b| {
                    a.bbox.min_y.partial_cmp(&b.bbox.min_y).unwrap()
                });
            }

            stack.push((l, mid));
            stack.push((mid, r));
        }
    }

    fn insert_recursive(
        node: &mut Entry,
        item: Entry,
        target_level: usize,
        max_entries: usize,
        min_entries: usize,
    ) -> Option<Entry> {
        node.bbox.extend(&item.bbox);

        if node.height - 1 == target_level {
            node.children.push(item);
            if node.children.len() > max_entries {
                return Some(RBush::split(node, min_entries));
            }
            return None;
        }

        let best_index = RBush::choose_subtree(node, &item.bbox);

        let split_node = RBush::insert_recursive(
            &mut node.children[best_index],
            item,
            target_level,
            max_entries,
            min_entries,
        );

        if let Some(new_child) = split_node {
            node.children.push(new_child);
            if node.children.len() > max_entries {
                return Some(RBush::split(node, min_entries));
            }
        }

        None
    }

    fn choose_subtree(node: &Entry, bbox: &Rect) -> usize {
        let mut best_index = 0;
        let mut min_enlargement = f64::INFINITY;
        let mut min_area = f64::INFINITY;

        for (i, child) in node.children.iter().enumerate() {
            let area = child.bbox.area();
            let enlargement = bbox.enlarged_area(&child.bbox) - area;

            if enlargement < min_enlargement {
                min_enlargement = enlargement;
                min_area = if area < min_area { area } else { min_area };
                best_index = i;
            } else if enlargement == min_enlargement {
                if area < min_area {
                    min_area = area;
                    best_index = i;
                }
            }
        }
        best_index
    }

    fn split(node: &mut Entry, min_entries: usize) -> Entry {
        let count = node.children.len();

        RBush::choose_split_axis(node, min_entries, count);
        let split_index = RBush::choose_split_index(node, min_entries, count);

        let new_children = node.children.split_off(split_index);
        let mut new_node = Entry::new_node(new_children);
        new_node.height = node.height;

        node.calc_bbox();
        new_node.calc_bbox();

        new_node
    }

    fn choose_split_axis(node: &mut Entry, m: usize, count: usize) {
        let x_margin = RBush::all_dist_margin(node, m, count, true);
        let y_margin = RBush::all_dist_margin(node, m, count, false);

        if x_margin < y_margin {
            node.children
                .sort_by(|a, b| a.bbox.min_x.partial_cmp(&b.bbox.min_x).unwrap());
        }
    }

    fn all_dist_margin(node: &mut Entry, m: usize, count: usize, compare_x: bool) -> f64 {
        if compare_x {
            node.children
                .sort_by(|a, b| a.bbox.min_x.partial_cmp(&b.bbox.min_x).unwrap());
        } else {
            node.children
                .sort_by(|a, b| a.bbox.min_y.partial_cmp(&b.bbox.min_y).unwrap());
        }

        let mut left_bbox = Rect::new_empty();
        let mut right_bbox = Rect::new_empty();

        for i in 0..m {
            left_bbox.extend(&node.children[i].bbox);
        }
        for i in (count - m)..count {
            right_bbox.extend(&node.children[i].bbox);
        }

        let mut margin = left_bbox.margin() + right_bbox.margin();

        for i in m..(count - m) {
            left_bbox.extend(&node.children[i].bbox);
            margin += left_bbox.margin();
        }
        for i in ((m)..(count - m)).rev() {
            right_bbox.extend(&node.children[i].bbox);
            margin += right_bbox.margin();
        }

        margin
    }

    fn choose_split_index(node: &Entry, m: usize, count: usize) -> usize {
        let mut min_overlap = f64::INFINITY;
        let mut min_area = f64::INFINITY;
        let mut index = count - m;

        for i in m..=(count - m) {
            let mut bbox1 = Rect::new_empty();
            let mut bbox2 = Rect::new_empty();

            for c in &node.children[0..i] {
                bbox1.extend(&c.bbox);
            }
            for c in &node.children[i..count] {
                bbox2.extend(&c.bbox);
            }

            let overlap = bbox1.intersection_area(&bbox2);
            let area = bbox1.area() + bbox2.area();

            if overlap < min_overlap {
                min_overlap = overlap;
                index = i;
                min_area = if area < min_area { area } else { min_area };
            } else if overlap == min_overlap {
                if area < min_area {
                    min_area = area;
                    index = i;
                }
            }
        }
        index
    }
}
