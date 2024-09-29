use std::collections::BTreeMap;

pub struct Tabs {
    tabs: BTreeMap<usize, Tabs>,
}
