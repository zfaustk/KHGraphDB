using System.Collections.Generic;

namespace KHGraphDB.Structure.Interface
{
    public interface IGraph : IDBObject
    {
        bool IsDirected { get; }

        IEnumerable<IVertex> Vertices { get; }

        IEnumerable<IEdge> Edges { get; }

        IEnumerable<IType> Types { get; }

        long VertexCount { get; }

        long EdgeCount { get; }

        long TypeCount { get; }

        IVertex AddVertex(IDictionary<string, object> attributes);

        IVertex AddVertex(IDictionary<string, object> attributes, IType theType);

        bool AddVertex(IVertex theVertex);

        bool AddVertex(IVertex theVertex, IType theType);

        bool RemoveVertex(IVertex theVertex);

        IType AddType(string name, IDictionary<string, object> attributes);

        bool AddType(IType theType);

        bool RemoveType(IType theType);

        IEdge AddEdge(IVertex theSource, IVertex theTarget);

        IEdge AddEdge(IVertex theSource, IVertex theTarget, IDictionary<string, object> attributes);

        bool AddEdge(IEdge theEdge);

        bool RemoveEdge(IEdge theEdge);

        IVertex GetVertex(string khid);

        IVertex GetVertexByName(string name);

        IEdge GetEdge(string khid);

        IType GetTypeByName(string name);
    }
}
