using System.Collections.Generic;
using KHGraphDB.Language;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Tests
{
    public static class QueryTests
    {
        static Graph Social()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            IType knows = g.AddType("KNOWS", null);
            g.CreateIndex("Person", "name");
            IVertex alice = P(g, person, "Alice");
            IVertex bob = P(g, person, "Bob");
            IVertex carol = P(g, person, "Carol");
            g.AddEdge(alice, bob, knows);
            g.AddEdge(bob, carol, knows);
            return g;
        }

        static IVertex P(Graph g, IType person, string name)
        {
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = name;
            return g.AddVertex(a, person);
        }

        public static void Nodes()
        {
            Query q = new Query(Social());
            QueryResult r = q.Run("MATCH (n:Person)");
            Assert.IsTrue(r.Succeeded, "match persons");
            Assert.Eq(3, r.Rows.Count, "three people");
        }

        public static void OneHop()
        {
            Query q = new Query(Social());
            QueryResult r = q.Run("MATCH (a:Person)-[:KNOWS]->(b:Person)");
            Assert.IsTrue(r.Succeeded, "one hop");
            Assert.Eq(2, r.Rows.Count, "Alice->Bob and Bob->Carol");
        }

        public static void Inbound()
        {
            Query q = new Query(Social());
            QueryResult r = q.Run("MATCH (a:Person)<-[:KNOWS]-(b:Person)");
            Assert.IsTrue(r.Succeeded, "inbound");
            Assert.Eq(2, r.Rows.Count, "Bob<-Alice and Carol<-Bob");
        }

        public static void Props()
        {
            Query q = new Query(Social());
            QueryResult r = q.Run("MATCH (a:Person {name:'Alice'})-[:KNOWS]->(b)");
            Assert.IsTrue(r.Succeeded, "props");
            Assert.Eq(1, r.Rows.Count, "only Alice");
            IVertex b = (IVertex)r.Rows[0][1];
            Assert.Eq("Bob", b["name"], "to Bob");
        }

        public static void TwoHop()
        {
            Query q = new Query(Social());
            QueryResult r = q.Run("MATCH (a:Person)-[:KNOWS]->(b)-[:KNOWS]->(c)");
            Assert.IsTrue(r.Succeeded, "two hop");
            Assert.Eq(1, r.Rows.Count, "Alice Bob Carol");
            Assert.Eq("Carol", ((IVertex)r.Rows[0][2])["name"], "end Carol");
        }

        public static void Cycle()
        {
            Graph g = Social();
            IType knows = g.GetTypeByName("KNOWS");
            IVertex alice = g.GetVertexByName("Alice");
            IVertex carol = g.GetVertexByName("Carol");
            g.AddEdge(carol, alice, knows);
            Query q = new Query(g);
            QueryResult r = q.Run("MATCH (a:Person)-[:KNOWS]->(b)-[:KNOWS]->(c)-[:KNOWS]->(d)");
            Assert.IsTrue(r.Succeeded, "cycle query");
            Assert.IsTrue(r.Rows.Count <= 6, "path uniqueness bounds the cycle");
        }

        public static void Where()
        {
            Query q = new Query(Social());
            QueryResult r = q.Run("MATCH (a:Person)-[:KNOWS]->(b) WHERE a.name = 'Alice'");
            Assert.IsTrue(r.Succeeded, "where");
            Assert.Eq(1, r.Rows.Count, "Alice only");
        }
    }
}