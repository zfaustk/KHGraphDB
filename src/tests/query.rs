//! query tests.

use super::super::Graph;
use super::common::{attrs, social};

#[test]
fn collect_neighbors() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person)-[:KNOWS]->(b) RETURN a, collect(b) AS ns");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
    let mut lens = Vec::new();
    for row in r.rows.iter() {
        lens.push(row[1].as_ref().and_then(|v| v.as_list()).unwrap().len());
    }
    lens.sort();
    assert_eq!(lens, vec![1, 1]);
}

#[test]
fn count_star() {
    let mut g = social();
    let r = super::super::query::run(&mut g, "MATCH (a:Person) RETURN count(a)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.rows[0][0].as_ref().and_then(|v| v.as_id()), Some("3"));
    let r2 = super::super::query::run(&mut g,
        "MATCH (a:Person)-[:KNOWS]->(b) RETURN a, count(b)");
    assert_eq!(r2.rows.len(), 2);
}

#[test]
fn create_comma() {
    let mut g = Graph::new();
    let r = super::super::query::run(&mut g,
        "CREATE (a:Person {name:'Ada'}), (b:Person {name:'Bob'}), (a)-[:KNOWS]->(b)");
    assert!(r.ok);
    assert_eq!(g.vertex_count(), 2);
    assert_eq!(g.edge_count(), 1);
    assert_eq!(r.columns.len(), 2);
}

#[test]
fn create_edge() {
    let mut g = Graph::new();
    let r = super::super::query::run(&mut g,
        "CREATE (a:Person {name:'Ada'})-[:KNOWS]->(b:Person {name:'Bob'})");
    assert!(r.ok);
    assert_eq!(g.vertex_count(), 2);
    assert_eq!(g.edge_count(), 1);
    assert_eq!(g.edges_of_type("KNOWS").len(), 1);
}

#[test]
fn create_node() {
    let mut g = Graph::new();
    let r = super::super::query::run(&mut g, "CREATE (n:Person {name:'Ada'})");
    assert!(r.ok);
    assert_eq!(r.created, 1);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(g.vertex_count(), 1);
    assert!(g.vertex_by_name("Ada").is_some());
    assert!(g.has_type(&g.vertex_by_name("Ada").unwrap().khid().to_string(), "Person"));
}

#[test]
fn delete_free_node() {
    let mut g = Graph::new();
    super::super::query::run(&mut g, "CREATE (n:Person {name:'Solo'})");
    let r = super::super::query::run(&mut g, "MATCH (n:Person {name:'Solo'}) DELETE n");
    assert!(r.ok);
    assert_eq!(g.vertex_count(), 0);
}

#[test]
fn delete_refuses_edges() {
    let mut g = social();
    let r = super::super::query::run(&mut g, "MATCH (n:Person {name:'Alice'}) DELETE n");
    assert!(!r.ok);
    assert_eq!(g.vertex_count(), 3);
}

#[test]
fn detach_delete() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (n:Person {name:'Alice'}) DETACH DELETE n");
    assert!(r.ok);
    assert_eq!(g.vertex_count(), 2);
    assert_eq!(g.edge_count(), 1);
}

#[test]
fn distinct_names() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person)-[:KNOWS]->(b) RETURN DISTINCT a");
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn explain_types() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "EXPLAIN MATCH (a:Person)-[:KNOWS]->(b)");
    assert!(r.ok);
    assert_eq!(r.columns, vec!["slot".to_string(), "name".to_string(), "khid".to_string()]);
    assert!(r.rows.len() >= 2);
    let person = g.type_by_name("Person").unwrap().khid().to_string();
    let mut saw = false;
    for row in r.rows.iter() {
        if row[1].as_ref().and_then(|v| v.as_id()) == Some("Person") {
            assert_eq!(row[2].as_ref().and_then(|v| v.as_id()), Some(person.as_str()));
            saw = true;
        }
    }
    assert!(saw);
}

#[test]
fn match_keyed_end() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person)-[:KNOWS]->(b:Person {name:'Bob'})");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.columns, vec!["a".to_string(), "b".to_string()]);
    let a = r.rows[0][0].as_ref().and_then(|v| v.as_id()).unwrap();
    assert_eq!(g.vertex(a).unwrap().get("name"), Some("Alice"));
    let b = r.rows[0][1].as_ref().and_then(|v| v.as_id()).unwrap();
    assert_eq!(g.vertex(b).unwrap().get("name"), Some("Bob"));
}

#[test]
fn match_keyed_end_path() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH p = (a:Person)-[:KNOWS]->(b:Person {name:'Bob'})");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    let p = r.rows[0][0].as_ref().and_then(|v| v.as_path()).unwrap();
    assert_eq!(p.hops(), 1);
    let a = g.vertex_by_name("Alice").unwrap().khid();
    let b = g.vertex_by_name("Bob").unwrap().khid();
    assert_eq!(p[0], a);
    assert_eq!(p[2], b);
}

#[test]
fn match_keyed_end_scan() {
    let mut g = Graph::new();
    let alice = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    g.add_edge(&alice, &bob, Some("KNOWS")).unwrap();
    assert!(!g.has_index("Person", "name"));
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person)-[:KNOWS]->(b:Person {name:'Bob'})");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    let a = r.rows[0][0].as_ref().and_then(|v| v.as_id()).unwrap();
    assert_eq!(a, alice);
}

#[test]
fn match_one_hop() {
    let mut g = social();
    let r = super::super::query::run(&mut g, "MATCH (a:Person)-[:KNOWS]->(b:Person)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn match_path_bind() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH p = (a:Person {name:'Alice'})-[:KNOWS*1..2]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
    assert_eq!(&r.columns[0], "p");
    let mut lens = Vec::new();
    let mut hops = Vec::new();
    for row in r.rows.iter() {
        let p = row[0].as_ref().and_then(|v| v.as_path()).unwrap();
        lens.push(p.len());
        hops.push(p.hops());
    }
    lens.sort();
    hops.sort();
    assert_eq!(lens, vec![3, 5]);
    assert_eq!(hops, vec![1, 2]);
}

#[test]
fn match_rel_bind() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[e:KNOWS]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.columns, vec!["a".to_string(), "e".to_string(), "b".to_string()]);
    let e = r.rows[0][1].as_ref().and_then(|v| v.as_id()).unwrap();
    let alice = g.vertex_by_name("Alice").unwrap().khid().to_string();
    assert_eq!(g.edge(e).unwrap().source(), super::super::Khid::parse(&alice).unwrap());
}

#[test]
fn match_rel_star_list() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[e:KNOWS*1..2]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
    let mut lens = Vec::new();
    for row in r.rows.iter() {
        let list = row[1].as_ref().and_then(|v| v.as_list()).unwrap();
        lens.push(list.len());
    }
    lens.sort();
    assert_eq!(lens, vec![1, 2]);
}

#[test]
fn match_shortest_hops() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let c = g.add_vertex(attrs("C"), Some("City")).unwrap();
    let ab = g.add_edge(&a, &b, Some("ROAD")).unwrap();
    let bc = g.add_edge(&b, &c, Some("ROAD")).unwrap();
    let ac = g.add_edge(&a, &c, Some("ROAD")).unwrap();
    g.set_edge_attr(&ab, "weight", "2");
    g.set_edge_attr(&bc, "weight", "2");
    g.set_edge_attr(&ac, "weight", "5");
    let r = super::super::query::run(&mut g,
        "MATCH p = shortestPath((x:City {name:'A'})-[:ROAD*]->(y:City {name:'C'}))");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    let p = r.rows[0][0].as_ref().and_then(|v| v.as_path()).unwrap();
    assert_eq!(p.len(), 3);
    assert_eq!(p.hops(), 1);
    assert_eq!(p[0], super::super::Khid::parse(&a).unwrap());
    assert_eq!(p[1], super::super::Khid::parse(&ac).unwrap());
    assert_eq!(p[2], super::super::Khid::parse(&c).unwrap());
    assert_eq!(p.nodes(),
               vec![super::super::Khid::parse(&a).unwrap(),
                    super::super::Khid::parse(&c).unwrap()]);
    assert_eq!(p.edges(), vec![super::super::Khid::parse(&ac).unwrap()]);
}

#[test]
fn match_shortest_none() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH p = shortestPath((a:Person {name:'Carol'})-[:KNOWS*]->(b:Person {name:'Alice'}))");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 0);
}

#[test]
fn match_star_cycle_lid() {
    let mut g = social();
    let alice = g.vertex_by_name("Alice").unwrap().khid().to_string();
    let carol = g.vertex_by_name("Carol").unwrap().khid().to_string();
    g.add_edge(&carol, &alice, Some("KNOWS")).unwrap();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[:KNOWS*1..8]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn match_star_exact() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[:KNOWS*2]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn match_star_hops() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[:KNOWS*1..2]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn match_star_zero() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[:KNOWS*0..1]->(b:Person)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn match_unknown_rel() {
    let mut g = social();
    let r = super::super::query::run(&mut g, "MATCH (a)-[:Nope]->(b)");
    assert!(!r.ok);
}

#[test]
fn match_unknown_type() {
    let mut g = social();
    let r = super::super::query::run(&mut g, "MATCH (n:Nope)");
    assert!(!r.ok);
}

#[test]
fn match_where() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
                             "MATCH (a:Person)-[:KNOWS]->(b) WHERE a.name = 'Alice'");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn merge_ada() {
    let mut g = social();
    let r = super::super::query::run(&mut g, "MERGE (p:Person {name:'Ada'})");
    assert!(r.ok);
    assert_eq!(r.message, "created");
    let r2 = super::super::query::run(&mut g, "MERGE (p:Person {name:'Ada'})");
    assert_eq!(r2.message, "exists");
    assert_eq!(g.vertex_count(), 4);
}

#[test]
fn merge_edge() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MERGE (a:Person {name:'Alice'})-[:KNOWS]->(b:Person {name:'Dan'})");
    assert!(r.ok);
    assert_eq!(g.vertex_count(), 4);
    assert_eq!(g.edge_count(), 3);
    let r2 = super::super::query::run(&mut g,
        "MERGE (a:Person {name:'Alice'})-[:KNOWS]->(b:Person {name:'Dan'})");
    assert_eq!(r2.message, "exists");
    assert_eq!(g.edge_count(), 3);
}

#[test]
fn merge_on_create() {
    let mut g = Graph::new();
    g.create_index("Person", "name");
    let r = super::super::query::run(&mut g,
        "MERGE (p:Person {name:'Ada'}) ON CREATE SET p.born = '1815'");
    assert!(r.ok);
    assert_eq!(g.vertex_by_name("Ada").unwrap().get("born"), Some("1815"));
    let r2 = super::super::query::run(&mut g,
        "MERGE (p:Person {name:'Ada'}) ON CREATE SET p.born = 'x' ON MATCH SET p.hit = '1'");
    assert_eq!(r2.message, "exists");
    let ada = g.vertex_by_name("Ada").unwrap();
    assert_eq!(ada.get("born"), Some("1815"));
    assert_eq!(ada.get("hit"), Some("1"));
}

#[test]
fn optional_nobody() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
                             "OPTIONAL MATCH (a:Person {name:'Nobody'})-[:KNOWS]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn order_by_name() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person) RETURN a ORDER BY a.name DESC");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 3);
    let first = r.rows[0][0].as_ref().and_then(|v| v.as_id()).unwrap();
    assert_eq!(g.vertex(first).unwrap().get("name"), Some("Carol"));
}

#[test]
fn parse_names_the_token() {
    let mut g = Graph::new();
    let r = super::super::query::run(&mut g, "MATCH (a:Person");
    assert!(!r.ok);
    assert!(r.message.contains("at end"));
    let r = super::super::query::run(&mut g, "MATCH (a) SET 1");
    assert!(!r.ok);
    assert!(r.message.contains("near 1"));
    let r = super::super::query::run(&mut g, "@nope");
    assert!(!r.ok);
    assert!(r.message.contains("near @"));
    let r = super::super::query::run(&mut g, "EXPLAIN CREATE (n)");
    assert!(!r.ok);
    assert!(r.message.contains("near CREATE"));
}

#[test]
fn path_functions() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH p = (a:Person {name:'Alice'})-[:KNOWS*1..2]->(b) RETURN length(p), nodes(p), relationships(p)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
    let mut hops = Vec::new();
    for row in r.rows.iter() {
        hops.push(row[0].as_ref().and_then(|v| v.as_id()).unwrap().to_string());
    }
    hops.sort();
    assert_eq!(hops, vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn remove_property() {
    let mut g = social();
    super::super::query::run(&mut g, "MATCH (a:Person {name:'Alice'}) SET a.city = 'London'");
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'}) REMOVE a.city");
    assert!(r.ok);
    let alice = g.vertex_by_name("Alice").unwrap();
    assert!(alice.get("city").is_none());
}

#[test]
fn return_as() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[:KNOWS]->(b) RETURN b AS friend");
    assert!(r.ok);
    assert_eq!(r.columns, vec!["friend".to_string()]);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn second_match() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[:KNOWS]->(b) MATCH (b)-[:KNOWS]->(c) RETURN a, b, c");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.columns.len(), 3);
    let c = r.rows[0][2].as_ref().and_then(|v| v.as_id()).unwrap();
    assert_eq!(g.vertex(c).unwrap().get("name"), Some("Carol"));
}

#[test]
fn set_edge() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[e:KNOWS]->(b) SET e.since = '2011'");
    assert!(r.ok);
    let eids = g.edges_of_type("KNOWS");
    let mut saw = false;
    for eid in eids.iter() {
        if g.edge(eid).unwrap().get("since") == Some("2011") {
            saw = true;
        }
    }
    assert!(saw);
}

#[test]
fn set_property() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'}) SET a.city = 'London'");
    assert!(r.ok);
    let alice = g.vertex_by_name("Alice").unwrap();
    assert_eq!(alice.get("city"), Some("London"));
}

#[test]
fn skip_limit() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person) RETURN a ORDER BY a.name SKIP 1 LIMIT 1");
    assert_eq!(r.rows.len(), 1);
    let id = r.rows[0][0].as_ref().and_then(|v| v.as_id()).unwrap();
    assert_eq!(g.vertex(id).unwrap().get("name"), Some("Bob"));
}

#[test]
fn unwind_list() {
    let mut g = Graph::new();
    let r = super::super::query::run(&mut g, "UNWIND ['Ada', 'Bob'] AS n RETURN n");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
    assert_eq!(r.rows[0][0].as_ref().and_then(|v| v.as_id()), Some("Ada"));
}

#[test]
fn unwind_star() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[e:KNOWS*1..2]->(b) UNWIND e AS hop RETURN hop");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn val_list() {
    let v = super::super::Val::List(vec![
        super::super::Val::Id("k1".to_string()),
        super::super::Val::Id("k2".to_string()),
    ]);
    assert_eq!(v.as_list().unwrap().len(), 2);
    assert!(v.as_id().is_none());
    assert!(v.as_path().is_none());
}

#[test]
fn path_is_khid() {
    let p = super::super::Path::parse_all(&[
        "k1".to_string(),
        "k2".to_string(),
        "k3".to_string(),
    ]);
    assert_eq!(p.hops(), 1);
    assert_eq!(p.len(), 3);
    assert_eq!(p[0], super::super::Khid::from_raw(1));
    assert_eq!(p[1], super::super::Khid::from_raw(2));
    assert_eq!(p.nodes(),
               vec![super::super::Khid::from_raw(1), super::super::Khid::from_raw(3)]);
    assert_eq!(p.edges(), vec![super::super::Khid::from_raw(2)]);
}

#[test]
fn where_compare() {
    let mut g = social();
    super::super::query::run(&mut g, "MATCH (a:Person {name:'Alice'}) SET a.age = 36");
    super::super::query::run(&mut g, "MATCH (a:Person {name:'Bob'}) SET a.age = 20");
    super::super::query::run(&mut g, "MATCH (a:Person {name:'Carol'}) SET a.age = 44");
    let r = super::super::query::run(&mut g, "MATCH (a:Person) WHERE a.age > 30");
    assert_eq!(r.rows.len(), 2);
    let r2 = super::super::query::run(&mut g, "MATCH (a:Person) WHERE a.age <= 20");
    assert_eq!(r2.rows.len(), 1);
}

#[test]
fn where_edge_attr() {
    let mut g = social();
    super::super::query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[e:KNOWS]->(b) SET e.weight = 3");
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person)-[e:KNOWS]->(b) WHERE e.weight > 1 RETURN e");
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn where_in() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person) WHERE a.name IN ['Alice', 'Carol']");
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn where_or_not() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person) WHERE a.name = 'Alice' OR a.name = 'Carol'");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 2);
    let r2 = super::super::query::run(&mut g,
        "MATCH (a:Person) WHERE NOT a.name = 'Alice'");
    assert_eq!(r2.rows.len(), 2);
}

#[test]
fn with_drops() {
    let mut g = social();
    let r = super::super::query::run(&mut g,
        "MATCH (a:Person)-[:KNOWS]->(b) WITH a WHERE a.name = 'Alice' RETURN a");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.columns, vec!["a".to_string()]);
}

#[test]
fn int_is_not_str_in_where() {
    let mut g = social();
    super::super::query::run(&mut g, "MATCH (a:Person {name:'Alice'}) SET a.age = 36");
    let r = super::super::query::run(&mut g, "MATCH (a:Person) WHERE a.age = 36");
    assert_eq!(r.rows.len(), 1);
    let r2 = super::super::query::run(&mut g, "MATCH (a:Person) WHERE a.age = '36'");
    assert_eq!(r2.rows.len(), 0);
    let r3 = super::super::query::run(&mut g, "MATCH (a:Person) WHERE a.age > '36'");
    assert_eq!(r3.rows.len(), 0);
}

#[test]
fn bool_prop() {
    let mut g = social();
    super::super::query::run(&mut g, "MATCH (a:Person {name:'Alice'}) SET a.alive = true");
    let r = super::super::query::run(&mut g, "MATCH (a:Person) WHERE a.alive = true");
    assert_eq!(r.rows.len(), 1);
    let r2 = super::super::query::run(&mut g, "MATCH (a:Person) WHERE a.alive = false");
    assert_eq!(r2.rows.len(), 0);
    let alice = g.vertex_by_name("Alice").unwrap();
    assert_eq!(alice.get_prop("alive").and_then(|p| p.as_bool()), Some(true));
    assert!(alice.get("alive").is_none());
}

#[test]
fn create_int_prop() {
    let mut g = Graph::new();
    let r = super::super::query::run(&mut g, "CREATE (a:Person {name:'Ada', born:1815})");
    assert!(r.ok);
    let ada = g.vertex_by_name("Ada").unwrap();
    assert_eq!(ada.get("name"), Some("Ada"));
    assert_eq!(ada.get_prop("born").and_then(|p| p.as_int()), Some(1815));
}

#[test]
fn param_str() {
    let mut g = social();
    let mut p = std::collections::HashMap::new();
    p.insert("n".to_string(), super::super::Prop::from_str("Alice"));
    let r = super::super::query::run_with(&mut g, "MATCH (a:Person {name:$n})", p);
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    let id = r.rows[0][0].as_ref().and_then(|v| v.as_id()).unwrap();
    assert_eq!(g.vertex(id).unwrap().get("name"), Some("Alice"));
}

#[test]
fn param_int_keeps_tag() {
    let mut g = social();
    super::super::query::run(&mut g, "MATCH (a:Person {name:'Alice'}) SET a.age = 36");
    let mut p = std::collections::HashMap::new();
    p.insert("x".to_string(), super::super::Prop::from_int(36));
    let r = super::super::query::run_with(&mut g, "MATCH (a:Person) WHERE a.age = $x", p);
    assert_eq!(r.rows.len(), 1);
    let mut p2 = std::collections::HashMap::new();
    p2.insert("x".to_string(), super::super::Prop::from_str("36"));
    let r2 = super::super::query::run_with(&mut g, "MATCH (a:Person) WHERE a.age = $x", p2);
    assert_eq!(r2.rows.len(), 0);
}

#[test]
fn param_unknown() {
    let mut g = Graph::new();
    let r = super::super::query::run(&mut g, "MATCH (a:Person {name:$n})");
    assert!(!r.ok);
    assert!(r.message.contains("unknown param $n"));
}

