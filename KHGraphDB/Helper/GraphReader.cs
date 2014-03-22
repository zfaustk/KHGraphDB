using System.Collections.Generic;
using System.IO;
using System.Text;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Helper
{
    public static class GraphReader
    {
        public static Graph Read(Stream stream)
        {
            if (stream == null)
                return null;

            BinaryReader r = new BinaryReader(stream, Encoding.UTF8);
            string magic = r.ReadString();
            if (magic == GraphWriter.Magic)
                return Read2(r);
            if (magic == GraphWriter.Magic1)
                return Read1(r);
            throw new InvalidDataException("not a KHGraphDB snapshot");
        }

        public static Graph ReadFile(string path)
        {
            using (FileStream fs = File.OpenRead(path))
                return Read(fs);
        }

        static Graph Read2(BinaryReader r)
        {
            string id = r.ReadString();
            bool directed = r.ReadBoolean();
            Graph g = new Graph(string.IsNullOrEmpty(id) ? null : id);
            if (!directed)
                return g;

            int nT = r.ReadInt32();
            for (int i = 0; i < nT; i++)
            {
                string khid = r.ReadString();
                string name = r.ReadString();
                IDictionary<string, object> attrs = ReadAttrs(r);
                Type t = new Type(khid, attrs);
                t.Name = name;
                g.AddType(t);
            }

            int nV = r.ReadInt32();
            for (int i = 0; i < nV; i++)
            {
                string khid = r.ReadString();
                int nNames = r.ReadInt32();
                List<string> names = new List<string>(nNames);
                for (int k = 0; k < nNames; k++)
                    names.Add(r.ReadString());
                IDictionary<string, object> attrs = ReadAttrs(r);
                Vertex v = new Vertex(khid, attrs);
                IType primary = names.Count > 0 ? g.GetTypeByName(names[0]) : null;
                g.AddVertex(v, primary);
                for (int k = 1; k < names.Count; k++)
                {
                    IType extra = g.GetTypeByName(names[k]);
                    if (extra != null)
                        v.AddType(extra);
                }
            }

            int nE = r.ReadInt32();
            for (int i = 0; i < nE; i++)
            {
                string khid = r.ReadString();
                string src = r.ReadString();
                string dst = r.ReadString();
                string typeName = r.ReadString();
                IDictionary<string, object> attrs = ReadAttrs(r);
                IVertex a = g.GetVertex(src);
                IVertex b = g.GetVertex(dst);
                if (a == null || b == null)
                    continue;
                Edge e = new Edge(khid, a, b, attrs);
                IType et = string.IsNullOrEmpty(typeName) ? null : g.GetTypeByName(typeName);
                if (et != null)
                    e.Type = et;
                g.AddEdge(e);
            }

            int nI = r.ReadInt32();
            for (int i = 0; i < nI; i++)
            {
                string typeName = r.ReadString();
                string key = r.ReadString();
                bool unique = r.ReadBoolean();
                if (unique)
                    g.CreateUniqueConstraint(typeName, key);
                else
                    g.CreateIndex(typeName, key);
            }
            return g;
        }

        static Graph Read1(BinaryReader r)
        {
            string id = r.ReadString();
            bool directed = r.ReadBoolean();
            Graph g = new Graph(string.IsNullOrEmpty(id) ? null : id);
            if (!directed)
                return g;

            int nV = r.ReadInt32();
            for (int i = 0; i < nV; i++)
            {
                string khid = r.ReadString();
                string typeName = r.ReadString();
                IDictionary<string, object> attrs = ReadAttrs(r);
                Vertex v = new Vertex(khid, attrs);
                IType t = null;
                if (!string.IsNullOrEmpty(typeName))
                    t = g.AddType(typeName, null);
                g.AddVertex(v, t);
            }

            int nE = r.ReadInt32();
            for (int i = 0; i < nE; i++)
            {
                string khid = r.ReadString();
                string src = r.ReadString();
                string dst = r.ReadString();
                IDictionary<string, object> attrs = ReadAttrs(r);
                IVertex a = g.GetVertex(src);
                IVertex b = g.GetVertex(dst);
                if (a == null || b == null)
                    continue;
                Edge e = new Edge(khid, a, b, attrs);
                g.AddEdge(e);
            }
            return g;
        }

        static IDictionary<string, object> ReadAttrs(BinaryReader r)
        {
            int n = r.ReadInt32();
            Dictionary<string, object> attrs = new Dictionary<string, object>(n);
            for (int i = 0; i < n; i++)
            {
                string key = r.ReadString();
                string val = r.ReadString();
                attrs[key] = val;
            }
            return attrs;
        }
    }
}
