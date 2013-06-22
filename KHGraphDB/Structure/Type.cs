using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Type : DBObject, IType
    {
        private IGraph _graph;
        private string _name;
        private HashSet<IVertex> _vertices;

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
        }

        public IGraph Graph
        {
            get { return _graph; }
            set { _graph = value; }
        }

        public string Name
        {
            get { return _name; }
            set { _name = value; }
        }

        public IEnumerable<IVertex> Vertices
        {
            get { return _vertices; }
        }

        public long VertexCount
        {
            get { return _vertices.Count; }
        }

        public bool AddVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return false;
            if (!_vertices.Add(theVertex))
                return false;
            theVertex.Type = this;
            return true;
        }

        public bool RemoveVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return false;
            if (!_vertices.Remove(theVertex))
                return false;
            if (object.ReferenceEquals(theVertex.Type, this))
                theVertex.Type = null;
            return true;
        }

        public void ClearVertices()
        {
            foreach (IVertex v in _vertices)
            {
                if (object.ReferenceEquals(v.Type, this))
                    v.Type = null;
            }
            _vertices.Clear();
        }
    }
}
