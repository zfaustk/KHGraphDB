//! BI-style reads. Count and collect. Not a
//! cost-based warehouse. The type is the scan.

use crate::graph::Graph;
use crate::query::{self, QueryResult};
use super::gen::Social;
use super::is::person_name;

pub fn bi1_count_people(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (a:Person) RETURN count(a)")
}

pub fn bi2_count_posts(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (p:Post) RETURN count(p)")
}

pub fn bi3_count_knows(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH ()-[e:KNOWS]->() RETURN count(e)")
}

pub fn bi4_count_likes(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH ()-[e:LIKES]->() RETURN count(e)")
}

pub fn bi5_people_in_city(g: &Graph, city: &str) -> QueryResult {
    let q = format!("MATCH (a:Person {{city:'{}'}}) RETURN count(a)", city);
    query::ask(g, &q)
}

pub fn bi6_posts_in_year(g: &Graph, year: i64) -> QueryResult {
    let q = format!("MATCH (p:Post {{year:{}}}) RETURN count(p)", year);
    query::ask(g, &q)
}

pub fn bi7_forums(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (f:Forum) RETURN f.title LIMIT 50")
}

pub fn bi8_tags(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (t:Tag) RETURN t.name LIMIT 50")
}

pub fn bi9_replies(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (r:Post)-[:REPLY_OF]->(p:Post) RETURN count(r)")
}

pub fn bi10_creators(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (p:Post)-[:HAS_CREATOR]->(a:Person) RETURN count(p)")
}

pub fn bi11_members(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (f:Forum)-[:HAS_MEMBER]->(a:Person) RETURN count(a)")
}

pub fn bi12_container(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (f:Forum)-[:CONTAINER_OF]->(p:Post) RETURN count(p)")
}

pub fn bi13_tagged_posts(g: &Graph) -> QueryResult {
    query::ask(g, "MATCH (p:Post)-[:HAS_TAG]->(t:Tag) RETURN count(p)")
}

pub fn bi14_person_posts(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (p:Post)-[:HAS_CREATOR]->(a:Person {{name:'{}'}}) RETURN count(p)",
        name
    );
    query::ask(g, &q)
}

pub fn run_all(s: &Social) -> Vec<(&'static str, QueryResult)> {
    let pn = person_name(s, 0);
    let city = s.g.vertex(s.people[0]).and_then(|v| v.get("city")).unwrap_or("London");
    vec![
        ("BI1", bi1_count_people(&s.g)),
        ("BI2", bi2_count_posts(&s.g)),
        ("BI3", bi3_count_knows(&s.g)),
        ("BI4", bi4_count_likes(&s.g)),
        ("BI5", bi5_people_in_city(&s.g, city)),
        ("BI6", bi6_posts_in_year(&s.g, 2015)),
        ("BI7", bi7_forums(&s.g)),
        ("BI8", bi8_tags(&s.g)),
        ("BI9", bi9_replies(&s.g)),
        ("BI10", bi10_creators(&s.g)),
        ("BI11", bi11_members(&s.g)),
        ("BI12", bi12_container(&s.g)),
        ("BI13", bi13_tagged_posts(&s.g)),
        ("BI14", bi14_person_posts(&s.g, &pn)),
    ]
}
