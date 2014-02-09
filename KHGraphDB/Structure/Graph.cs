using System;
using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Graph : DBObject, IGraph
    {
        private Dictionary<string, IVertex> _vertices;
        private Dictionary<string, IEdge> _edges;
        private Dictionary<string, IType> _types;
        private Dictionary<string, IType> _typesByName;
        private Dictionary<string, IVertex> _verticesByName;
        private Dictionary<string, SchemaIndex> _indexes;

        public Graph()
            : this(null, null)
        {
        }

        public Graph(string id)
            : this(id, null)
        {
        }

        public Graph(IDictionary<string, object> attributes)
            : this(null, attributes)
        {
        }

        public Graph(string id, IDictionary<string, object> attributes)
        {
            InitDBObject(id, attributes);
            _vertices = new Dictionary<string, IVertex>(StringComparer.Ordinal);
            _edges = new Dictionary<string, IEdge>(StringComparer.Ordinal);
            _types = new Dictionary<string, IType>(StringComparer.Ordinal);
            _typesByName = new Dictionary<string, IType>(StringComparer.Ordinal);
            _verticesByName = new Dictionary<string, IVertex>(StringComparer.Ordinal);
            _indexes = new Dictionary<string, SchemaIndex>(StringComparer.Ordinal);
        }

        public bool IsDirected
        {
            get { return true; }
        }

        public IEnumerable<IVertex> Vertices
        {
            get { return _vertices.Values; }
        }

        public IEnumerable<IEdge> Edges
        {
            get { return _edges.Values; }
        }

        public IEnumerable<IType> Types
        {
            get { return _types.Values; }
        }

        public long VertexCount
        {
            get { return _vertices.Count; }
        }

        public long EdgeCount
        {
            get { return _edges.Count; }
        }

        public long TypeCount
        {
            get { return _types.Count; }
        }

        public IVertex GetVertex(string khid)
        {
            if (khid == null)
                return null;
            IVertex v;
            if (_vertices.TryGetValue(khid, out v))
                return v;
            return null;
        }

        public IVertex GetVertexByName(string name)
        {
            if (name == null)
                return null;
            IVertex v;
            if (_verticesByName.TryGetValue(name, out v))
                return v;
            return null;
        }

        public IEdge GetEdge(string khid)
        {
            if (khid == null)
                return null;
            IEdge e;
            if (_edges.TryGetValue(khid, out e))
                return e;
            return null;
        }

        public IType GetTypeByName(string name)
        {
            if (name == null)
                return null;
            IType t;
            if (_typesByName.TryGetValue(name, out t))
                return t;
            return null;
        }

        public IEnumerable<IVertex> GetVerticesByType(string name)
        {
            IType t = GetTypeByName(name);
            if (t == null)
                return new IVertex[0];
            return t.Vertices;
        }

        public IEnumerable<IEdge> GetEdgesByType(string name)
        {
            IType t = GetTypeByName(name);
            if (t == null)
                return new IEdge[0];
            return t.Edges;
        }

        public IVertex AddVertex(IDictionary<string, object> attributes)
        {
            return AddVertex(attributes, null);
        }

        public IVertex AddVertex(IDictionary<string, object> attributes, IType theType)
        {
            Vertex v = new Vertex(attributes);
            if (AddVertex(v, theType))
                return v;
            return null;
        }

        public bool AddVertex(IVertex theVertex)
        {
            return AddVertex(theVertex, null);
        }

        public bool AddVertex(IVertex theVertex, IType theType)
        {
            if (theVertex == null)
                return false;

            IVertex existing;
            if (_vertices.TryGetValue(theVertex.KHID, out existing))
            {
                if (!object.ReferenceEquals(existing, theVertex))
                    return false;
                if (theType != null)
                    theType.AddVertex(theVertex);
                return true;
            }

            _vertices.Add(theVertex.KHID, theVertex);
            theVertex.Graph = this;
            IndexName(theVertex, theVertex[Vertex.NameKey]);
            IndexVertex(theVertex);
            if (theType != null)
            {
                if (theType.Graph == null)
                    AddType(theType);
                theType.AddVertex(theVertex);
            }
            return true;
        }

        public bool RemoveVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return false;
            IVertex owned;
            if (!_vertices.TryGetValue(theVertex.KHID, out owned))
                return false;
            if (!object.ReferenceEquals(owned, theVertex))
                return false;

            IEdge[] outgoing = new IEdge[theVertex.OutDegree];
            int n = 0;
            foreach (IEdge e in theVertex.OutgoingEdges)
                outgoing[n++] = e;
            IEdge[] incoming = new IEdge[theVertex.InDegree];
            n = 0;
            foreach (IEdge e in theVertex.IncomingEdges)
                incoming[n++] = e;
            for (int i = 0; i < outgoing.Length; i++)
                RemoveEdge(outgoing[i]);
            for (int i = 0; i < incoming.Length; i++)
                RemoveEdge(incoming[i]);

            List<IType> worn = new List<IType>();
            foreach (IType t in theVertex.Types)
                worn.Add(t);
            for (int i = 0; i < worn.Count; i++)
                worn[i].RemoveVertex(theVertex);

            UnindexVertex(theVertex);
            UnindexName(theVertex, theVertex[Vertex.NameKey]);
            theVertex.Graph = null;
            return _vertices.Remove(theVertex.KHID);
        }

        public IType AddType(string name, IDictionary<string, object> attributes)
        {
            if (string.IsNullOrEmpty(name))
                return null;
            IType existing = GetTypeByName(name);
            if (existing != null)
                return existing;
            Type t = new Type(attributes);
            t.Name = name;
            if (AddType(t))
                return t;
            return null;
        }

        public bool AddType(IType theType)
        {
            if (theType == null || string.IsNullOrEmpty(theType.Name))
                return false;

            IType byId;
            if (_types.TryGetValue(theType.KHID, out byId))
                return object.ReferenceEquals(byId, theType);

            IType byName = GetTypeByName(theType.Name);
            if (byName != null && !object.ReferenceEquals(byName, theType))
                return false;

            _types.Add(theType.KHID, theType);
            _typesByName[theType.Name] = theType;
            theType.Graph = this;
            return true;
        }

        public bool RemoveType(IType theType)
        {
            if (theType == null)
                return false;
            IType owned;
            if (!_types.TryGetValue(theType.KHID, out owned))
                return false;
            if (!object.ReferenceEquals(owned, theType))
                return false;
            theType.ClearVertices();
            theType.ClearEdges();
            theType.Graph = null;
            if (theType.Name != null)
                _typesByName.Remove(theType.Name);
            return _types.Remove(theType.KHID);
        }

        public IEdge AddEdge(IVertex theSource, IVertex theTarget)
        {
            return AddEdge(theSource, theTarget, null, null);
        }

        public IEdge AddEdge(IVertex theSource, IVertex theTarget, IDictionary<string, object> attributes)
        {
            return AddEdge(theSource, theTarget, null, attributes);
        }

        public IEdge AddEdge(IVertex theSource, IVertex theTarget, IType theType)
        {
            return AddEdge(theSource, theTarget, theType, null);
        }

        public IEdge AddEdge(IVertex theSource, IVertex theTarget, IType theType, IDictionary<string, object> attributes)
        {
            if (theSource == null || theTarget == null)
                return null;
            Edge e = new Edge(theSource, theTarget, attributes);
            if (theType != null)
                e.Type = theType;
            if (AddEdge(e))
                return e;
            return null;
        }

        public bool AddEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            if (!_vertices.ContainsKey(theEdge.Source.KHID) || !_vertices.ContainsKey(theEdge.Target.KHID))
                return false;

            IEdge existing;
            if (_edges.TryGetValue(theEdge.KHID, out existing))
                return object.ReferenceEquals(existing, theEdge);

            if (!theEdge.Source.AddOutgoingEdge(theEdge))
                return false;
            if (!theEdge.Target.AddIncomingEdge(theEdge))
            {
                theEdge.Source.RemoveOutgoingEdge(theEdge);
                return false;
            }
            _edges.Add(theEdge.KHID, theEdge);
            theEdge.Graph = this;
            if (theEdge.Type != null)
            {
                if (theEdge.Type.Graph == null)
                    AddType(theEdge.Type);
                theEdge.Type.AddEdge(theEdge);
            }
            return true;
        }

        public bool RemoveEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            IEdge owned;
            if (!_edges.TryGetValue(theEdge.KHID, out owned))
                return false;
            if (!object.ReferenceEquals(owned, theEdge))
                return false;
            theEdge.Source.RemoveOutgoingEdge(theEdge);
            theEdge.Target.RemoveIncomingEdge(theEdge);
            if (theEdge.Type != null)
                theEdge.Type.RemoveEdge(theEdge);
            theEdge.Graph = null;
            return _edges.Remove(theEdge.KHID);
        }


        public bool CreateIndex(string typeName, string key)
        {
            if (string.IsNullOrEmpty(typeName) || string.IsNullOrEmpty(key))
                return false;
            string id = SchemaIndex.Id(typeName, key);
            if (_indexes.ContainsKey(id))
                return true;
            SchemaIndex idx = new SchemaIndex(typeName, key, false);
            _indexes[id] = idx;
            IType t = GetTypeByName(typeName);
            if (t != null)
            {
                foreach (IVertex v in t.Vertices)
                    idx.Add(v, v[key]);
            }
            return true;
        }

        public IList<IVertex> Find(string typeName, string key, object value)
        {
            string id = SchemaIndex.Id(typeName, key);
            SchemaIndex idx;
            if (_indexes.TryGetValue(id, out idx))
                return idx.Get(value);

            List<IVertex> hits = new List<IVertex>();
            IType t = GetTypeByName(typeName);
            if (t == null)
                return hits;
            string want = SchemaIndex.ValueString(value);
            foreach (IVertex v in t.Vertices)
            {
                if (string.Equals(SchemaIndex.ValueString(v[key]), want, StringComparison.Ordinal))
                    hits.Add(v);
            }
            return hits;
        }


        internal void OnTypeAttached(IVertex theVertex, IType theType)
        {
            IndexVertexType(theVertex, theType);
        }

        internal void OnTypeDetached(IVertex theVertex, IType theType)
        {
            UnindexVertexType(theVertex, theType);
        }

        internal bool CanSetAttribute(IVertex theVertex, string key, object newValue)
        {
            if (theVertex == null || key == null)
                return true;
            foreach (IType t in theVertex.Types)
            {
                if (t.Name == null)
                    continue;
                SchemaIndex idx;
                if (!_indexes.TryGetValue(SchemaIndex.Id(t.Name, key), out idx))
                    continue;
                if (!idx.Unique)
                    continue;
                if (idx.ContainsOther(newValue, theVertex))
                    return false;
            }
            return true;
        }

        internal void OnVertexAttributeChanged(IVertex theVertex, string key, object oldValue, object newValue)
        {
            if (key == Vertex.NameKey)
                OnVertexNameChanged(theVertex, oldValue, newValue);
            if (theVertex == null || key == null)
                return;
            foreach (IType t in theVertex.Types)
            {
                if (t.Name == null)
                    continue;
                SchemaIndex idx;
                if (!_indexes.TryGetValue(SchemaIndex.Id(t.Name, key), out idx))
                    continue;
                idx.Remove(theVertex, oldValue);
                idx.Add(theVertex, newValue);
            }
        }

        void IndexVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return;
            foreach (IType t in theVertex.Types)
                IndexVertexType(theVertex, t);
        }

        void UnindexVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return;
            foreach (IType t in theVertex.Types)
                UnindexVertexType(theVertex, t);
        }

        void IndexVertexType(IVertex theVertex, IType theType)
        {
            if (theType == null || theType.Name == null)
                return;
            foreach (KeyValuePair<string, SchemaIndex> kv in _indexes)
            {
                SchemaIndex idx = kv.Value;
                if (idx.TypeName != theType.Name)
                    continue;
                idx.Add(theVertex, theVertex[idx.Key]);
            }
        }

        void UnindexVertexType(IVertex theVertex, IType theType)
        {
            if (theType == null || theType.Name == null)
                return;
            foreach (KeyValuePair<string, SchemaIndex> kv in _indexes)
            {
                SchemaIndex idx = kv.Value;
                if (idx.TypeName != theType.Name)
                    continue;
                idx.Remove(theVertex, theVertex[idx.Key]);
            }
        }

        internal void OnVertexNameChanged(IVertex theVertex, object oldName, object newName)
        {
            UnindexName(theVertex, oldName);
            IndexName(theVertex, newName);
        }

        void IndexName(IVertex theVertex, object nameObj)
        {
            string name = NameString(nameObj);
            if (name == null)
                return;
            _verticesByName[name] = theVertex;
        }

        void UnindexName(IVertex theVertex, object nameObj)
        {
            string name = NameString(nameObj);
            if (name == null)
                return;
            IVertex owned;
            if (_verticesByName.TryGetValue(name, out owned) && object.ReferenceEquals(owned, theVertex))
                _verticesByName.Remove(name);
        }

        static string NameString(object nameObj)
        {
            if (nameObj == null)
                return null;
            string s = nameObj.ToString();
            if (s.Length == 0)
                return null;
            return s;
        }
    }
}
