using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Vertex : DBObject, IVertex
    {
        private IGraph _graph;
        private IType _type;
        private HashSet<IEdge> _outgoing;
        private HashSet<IEdge> _incoming;

        public Vertex()
            : this(null, null)
        {
        }

        public Vertex(string id)
            : this(id, null)
        {
        }

        public Vertex(IDictionary<string, object> attributes)
            : this(null, attributes)
        {
        }

        public Vertex(string id, IDictionary<string, object> attributes)
        {
            InitDBObject(id, attributes);
            _outgoing = new HashSet<IEdge>();
            _incoming = new HashSet<IEdge>();
        }

        public IGraph Graph
        {
            get { return _graph; }
            set { _graph = value; }
        }

        public IType Type
        {
            get { return _type; }
            set { _type = value; }
        }

        public IEnumerable<IEdge> OutgoingEdges
        {
            get { return _outgoing; }
        }

        public IEnumerable<IEdge> IncomingEdges
        {
            get { return _incoming; }
        }

        public long Degree
        {
            get { return _outgoing.Count + _incoming.Count; }
        }

        public long InDegree
        {
            get { return _incoming.Count; }
        }

        public long OutDegree
        {
            get { return _outgoing.Count; }
        }

        public bool AddOutgoingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            return _outgoing.Add(theEdge);
        }

        public bool AddIncomingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            return _incoming.Add(theEdge);
        }

        public bool RemoveOutgoingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            return _outgoing.Remove(theEdge);
        }

        public bool RemoveIncomingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            return _incoming.Remove(theEdge);
        }
    }
}
