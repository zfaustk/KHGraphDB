//! Interactive updates. IU1 add person. IU2 add
//! KNOWS. IU3 add post. IU4 like. IU5 membership.

use std::collections::HashMap;
use crate::graph::Graph;
use crate::khid::Khid;
use crate::prop::Prop;
use crate::error::Result;

pub fn iu1(g: &mut Graph, name: &str, first: &str, last: &str, city: &str, year: i64) -> Result<Khid> {
    let mut a = HashMap::new();
    a.insert("name".to_string(), Prop::from_str(name));
    a.insert("first".to_string(), Prop::from_str(first));
    a.insert("last".to_string(), Prop::from_str(last));
    a.insert("city".to_string(), Prop::from_str(city));
    a.insert("year".to_string(), Prop::from_int(year));
    g.add_vertex_props(a, Some("Person"))
}

pub fn iu2(g: &mut Graph, a: Khid, b: Khid) -> Result<Khid> {
    g.add_edge(a, b, Some("KNOWS"))
}

pub fn iu3(g: &mut Graph, title: &str, year: i64, author: Khid) -> Result<Khid> {
    let mut a = HashMap::new();
    a.insert("name".to_string(), Prop::from_str(title));
    a.insert("title".to_string(), Prop::from_str(title));
    a.insert("year".to_string(), Prop::from_int(year));
    let p = g.add_vertex_props(a, Some("Post"))?;
    g.add_edge(p, author, Some("HAS_CREATOR"))?;
    Ok(p)
}

pub fn iu4(g: &mut Graph, person: Khid, post: Khid) -> Result<Khid> {
    g.add_edge(person, post, Some("LIKES"))
}

pub fn iu5(g: &mut Graph, forum: Khid, person: Khid) -> Result<Khid> {
    g.add_edge(forum, person, Some("HAS_MEMBER"))
}

pub fn iu6(g: &mut Graph, forum: Khid, post: Khid) -> Result<Khid> {
    g.add_edge(forum, post, Some("CONTAINER_OF"))
}

pub fn iu7(g: &mut Graph, reply: Khid, parent: Khid) -> Result<Khid> {
    g.add_edge(reply, parent, Some("REPLY_OF"))
}

pub fn iu8(g: &mut Graph, post: Khid, tag: Khid) -> Result<Khid> {
    g.add_edge(post, tag, Some("HAS_TAG"))
}
