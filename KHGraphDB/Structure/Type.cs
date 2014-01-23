using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Type : DBObject, IType
    {
        private IGraph _graph;
        private string _name;
        private HashSet<IVertex> _vertices;
        private HashSet<IEdge> _edges;

        public Type()
            : this(null, null)
        {
        }

        public Type(string id)
            : this(id, null)
        {
        }

        public Type(IDictionary<string, object> attributes)
            : this(null, attributes)
        {
        }

        public Type(string id, IDictionary<string, object> attributes)
        {
            InitDBObject(id, attributes);
            _vertices = new HashSet<IVertex>();
            _edges = new HashSet<IEdge>();
        }

        public IGraph Graph
        {
            get { return _graph; }
            set { _graph = value; }
        }

        public string Name
        {
            get { return _name; }
            set
            {
                if (_graph != null)
                    return;
                _name = value;
            }
        }

        public IEnumerable<IVertex> Vertices
        {
            get { return _vertices; }
        }

        public IEnumerable<IEdge> Edges
        {
            get { return _edges; }
        }

        public long VertexCount
        {
            get { return _vertices.Count; }
        }

        public long EdgeCount
        {
            get { return _edges.Count; }
        }

        public bool AddVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return false;
            Vertex v = theVertex as Vertex;
            if (v == null)
                return false;
            if (!_vertices.Add(theVertex))
                return false;
            v.AttachType(this);
            return true;
        }

        public bool RemoveVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return false;
            if (!_vertices.Remove(theVertex))
                return false;
            Vertex v = theVertex as Vertex;
            if (v != null)
                v.DetachType(this);
            return true;
        }

        public bool AddEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            if (!_edges.Add(theEdge))
                return false;
            if (theEdge.Type == null)
                theEdge.Type = this;
            return true;
        }

        public bool RemoveEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            if (!_edges.Remove(theEdge))
                return false;
            if (object.ReferenceEquals(theEdge.Type, this))
                theEdge.Type = null;
            return true;
        }

        public void ClearVertices()
        {
            List<IVertex> copy = new List<IVertex>(_vertices);
            _vertices.Clear();
            for (int i = 0; i < copy.Count; i++)
            {
                Vertex v = copy[i] as Vertex;
                if (v != null)
                    v.DetachType(this);
            }
        }

        public void ClearEdges()
        {
            List<IEdge> copy = new List<IEdge>(_edges);
            _edges.Clear();
            for (int i = 0; i < copy.Count; i++)
            {
                if (object.ReferenceEquals(copy[i].Type, this))
                    copy[i].Type = null;
            }
        }
    }
}
