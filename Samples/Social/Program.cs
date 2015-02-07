using System;
using System.Collections.Generic;
using KHGraphDB.Language;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Samples.Social
{
    public static class Program
    {
        public static void Main(string[] args)
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            IType knows = g.AddType("KNOWS", null);
            g.CreateUniqueConstraint("Person", "name");

            IVertex alice = AddPerson(g, person, "Alice");
            IVertex bob = AddPerson(g, person, "Bob");
            IVertex carol = AddPerson(g, person, "Carol");
            g.AddEdge(alice, bob, knows);
            g.AddEdge(bob, carol, knows);

            Command cmd = new Command(g);
            cmd.Run("MERGE (p:Person {name:'Ada'})");
            cmd.Run("MERGE (a:Person {name:'Alice'})-[:KNOWS]->(b:Person {name:'Ada'})");
            CommandResult r = cmd.Run("MATCH (a:Person)-[:KNOWS]->(b) WHERE a.name = 'Alice' RETURN b");
            Console.WriteLine(r.Message);
            for (int i = 0; i < r.Vertices.Count; i++)
                Console.WriteLine("  " + r.Vertices[i]["name"]);
        }

        static IVertex AddPerson(Graph g, IType person, string name)
        {
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = name;
            return g.AddVertex(a, person);
        }
    }
}
