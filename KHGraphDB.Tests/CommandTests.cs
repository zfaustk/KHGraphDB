using System.Collections.Generic;
using KHGraphDB.Language;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Tests
{
    public static class CommandTests
    {
        public static void Run()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            IType knows = g.AddType("KNOWS", null);
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = "Alice";
            Dictionary<string, object> b = new Dictionary<string, object>();
            b["name"] = "Bob";
            IVertex alice = g.AddVertex(a, person);
            IVertex bob = g.AddVertex(b, person);
            g.AddEdge(alice, bob, knows);

            Command cmd = new Command(g);
            CommandResult near = cmd.Run("near Alice 1");
            Assert.IsTrue(near.Succeeded, "near still works");
            CommandResult match = cmd.Run("MATCH (p:Person)-[:KNOWS]->(q) WHERE p.name = 'Alice'");
            Assert.IsTrue(match.Succeeded, "Command routes MATCH");
            Assert.IsTrue(match.Vertices.Count >= 1, "MATCH vertices");
        }
    }
}
