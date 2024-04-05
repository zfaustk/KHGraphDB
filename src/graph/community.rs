impl Graph {
    /// A community is a vertex. Members hop IN.
    /// It is not a view. Nest is another IN.
    pub fn community(&mut self) -> Result<Khid> {
        self.community_as("")
    }

    pub fn community_as(&mut self, name: &str) -> Result<Khid> {
        let _ = self.add_type("Community")?;
        let mut attrs = HashMap::new();
        if !name.is_empty() {
            attrs.insert("name".to_string(), name.to_string());
        }
        self.add_vertex(attrs, Some("Community"))
    }
}
