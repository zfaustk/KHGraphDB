//! LDBC SNB Interactive complex, cut to what a
//! notebook kernel can say in Cypher. IC1 friends
//! of friends by last name. IC2 recent posts of
//! friends. IC8 replies. IC13 two-hop reach.

use crate::graph::Graph;
use crate::query::{self, QueryResult};
use super::gen::Social;
use super::is::{person_name, post_title};

/// IC1: 2-hop KNOWS, filter last name.
pub fn ic1(g: &Graph, name: &str, last: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS]->(:Person)-[:KNOWS]->(c:Person {{last:'{}'}}) RETURN c.name LIMIT 20",
        name, last
    );
    query::ask(g, &q)
}

/// IC2: posts created by friends.
pub fn ic2(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS]->(b:Person)<-[:HAS_CREATOR]-(p:Post) RETURN p.title LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IC3: friends in a city.
pub fn ic3(g: &Graph, name: &str, city: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS]->(b:Person {{city:'{}'}}) RETURN b.name LIMIT 20",
        name, city
    );
    query::ask(g, &q)
}

/// IC4: tags on friends' posts. Walk only.
pub fn ic4(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS]->(:Person)<-[:HAS_CREATOR]-(p:Post)-[:HAS_TAG]->(t:Tag) RETURN t.name LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IC5: forums a person's friends belong to.
pub fn ic5(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS]->(b:Person)<-[:HAS_MEMBER]-(f:Forum) RETURN f.title LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IC6: tags of posts a person liked.
pub fn ic6(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:LIKES]->(p:Post)-[:HAS_TAG]->(t:Tag) RETURN t.name LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IC7: people who like a person's posts.
pub fn ic7(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (b:Person)-[:LIKES]->(p:Post)-[:HAS_CREATOR]->(a:Person {{name:'{}'}}) RETURN b.name LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IC8: replies to a person's posts.
pub fn ic8(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (r:Post)-[:REPLY_OF]->(p:Post)-[:HAS_CREATOR]->(a:Person {{name:'{}'}}) RETURN r.title LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IC9: posts of friends of friends.
pub fn ic9(g: &Graph, name: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS]->(:Person)-[:KNOWS]->(c:Person)<-[:HAS_CREATOR]-(p:Post) RETURN p.title LIMIT 20",
        name
    );
    query::ask(g, &q)
}

/// IC11: friends from a city, with year.
pub fn ic11(g: &Graph, name: &str, city: &str) -> QueryResult {
    ic3(g, name, city)
}

/// IC13: 2-hop path exists between two people.
pub fn ic13(g: &Graph, a: &str, b: &str) -> QueryResult {
    let q = format!(
        "MATCH (a:Person {{name:'{}'}})-[:KNOWS*1..2]->(b:Person {{name:'{}'}}) RETURN a LIMIT 1",
        a, b
    );
    query::ask(g, &q)
}

pub fn run_all(s: &Social) -> Vec<(&'static str, QueryResult)> {
    let pn = person_name(s, 0);
    let last = s.g.vertex(s.people[0]).and_then(|v| v.get("last")).unwrap_or("Smith");
    let city = s.g.vertex(s.people[0]).and_then(|v| v.get("city")).unwrap_or("London");
    let other = person_name(s, 1);
    let _ = post_title(s, 0);
    vec![
        ("IC1", ic1(&s.g, &pn, last)),
        ("IC2", ic2(&s.g, &pn)),
        ("IC3", ic3(&s.g, &pn, city)),
        ("IC4", ic4(&s.g, &pn)),
        ("IC5", ic5(&s.g, &pn)),
        ("IC6", ic6(&s.g, &pn)),
        ("IC7", ic7(&s.g, &pn)),
        ("IC8", ic8(&s.g, &pn)),
        ("IC9", ic9(&s.g, &pn)),
        ("IC11", ic11(&s.g, &pn, city)),
        ("IC13", ic13(&s.g, &pn, &other)),
    ]
}
