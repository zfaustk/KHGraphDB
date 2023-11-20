impl Graph {
    /// Recipe on the Type. Does not fill members.
    pub fn mark_view(&mut self, type_name: &str, query: &str) -> bool {
        if type_name.is_empty() || query.is_empty() {
            return false;
        }
        let tid = match self.add_type(type_name) {
            Ok(id) => id,
            Err(_) => return false,
        };
        match self.tget_mut(tid) {
            Some(t) => {
                t.mark_view(query);
            }
            None => return false,
        }
        self.rec(Touch::View {
            type_name: type_name.to_string(),
            query: query.to_string(),
        });
        true
    }

    pub fn view_of(&self, type_name: &str) -> Option<&str> {
        self.type_by_name(type_name).and_then(|t| t.view())
    }

    pub fn view_hash_of(&self, type_name: &str) -> Option<u64> {
        match self.type_by_name(type_name) {
            Some(t) if t.is_view() => Some(t.view_hash()),
            _ => None,
        }
    }

    pub fn is_view(&self, type_name: &str) -> bool {
        self.type_by_name(type_name).map(|t| t.is_view()).unwrap_or(false)
    }
}
