//! LDBC SNB Interactive short reads, against the
//! kernel Cypher. IS1 person, IS2 posts of person,
//! IS3 friends, IS4 post, IS5 forum of post, IS6
//! author, IS7 likes.

use crate::graph::Graph;
use crate::khid::Khid;
use crate::query::{self, QueryResult};
use super::gen::Social;

fn name_of(g: &Graph, id: Khid) -> String {
    g.vertex(id).and_then(|v| v.get("name")).unwrap_or("").to_string()
}

/// IS1: properties of a person by name.
pub fn is1(g: &Graph, name: &str) -> QueryResult {
    let q = format!("MATCH (a:Person {{name:'{}'}}) RETURN a.first, a.last, a.city, a.year", name);
    query::ask(g, &q)
}

/// IS2: posts this person created.
pub fn is2(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (p:Post)-[:HAS_CREATOR]->(a:Person {{name:'{}'}}) RETURN p.title LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IS3: people this person knows.
pub fn is3(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS]->(b:Person) RETURN b.name LIMIT 50",
        name
    );
    query::ask(g, &q)
}

/// IS4: a post by title.
pub fn is4(g: &Graph, title: &str) -> QueryResult {
    let q = format!("MATCH (p:Post {{name:'{}'}}) RETURN p.title, p.year", title);
    query::ask(g, &q)
}

/// IS5: forum that contains the post.
pub fn is5(g: &Graph, title: &str) -> QueryResult {
    let q = format!(
        "MATCH (f:Forum)-[:CONTAINER_OF]->(p:Post {{name:'{}'}}) RETURN f.title",
        title
    );
    query::ask(g, &q)
}

/// IS6: author of the post.
pub fn is6(g: &Graph, title: &str) -> QueryResult {
    let q = format!(
        "MATCH (p:Post {{name:'{}'}})-[:HAS_CREATOR]->(a:Person) RETURN a.name, a.city",
        title
    );
    query::ask(g, &q)
}

/// IS7: who liked the post.
pub fn is7(g: &Graph, title: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person)-[:LIKES]->(p:Post {{name:'{}'}}) RETURN a.name LIMIT 50",
        title
    );
    query::ask(g, &q)
}

pub fn person_name(s: &Social, i: usize) -> String {
    name_of(&s.g, s.people[i % s.people.len()])
}

pub fn post_title(s: &Social, i: usize) -> String {
    name_of(&s.g, s.posts[i % s.posts.len()])
}

pub fn run_all(s: &Social) -> Vec<(&'static str, QueryResult)> {
    let pn = person_name(s, 0);
    let pt = post_title(s, 0);
    vec![
        ("IS1", is1(&s.g, &pn)),
        ("IS2", is2(&s.g, &pn)),
        ("IS3", is3(&s.g, &pn)),
        ("IS4", is4(&s.g, &pt)),
        ("IS5", is5(&s.g, &pt)),
        ("IS6", is6(&s.g, &pt)),
        ("IS7", is7(&s.g, &pt)),
    ]
}
