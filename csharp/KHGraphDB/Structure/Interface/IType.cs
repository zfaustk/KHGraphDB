using System.Collections.Generic;

namespace KHGraphDB.Structure.Interface
{
    public interface IType : IDBObject
    {
        IGraph Graph { get; set; }

        string Name { get; set; }

        IEnumerable<IVertex> Vertices { get; }

        IEnumerable<IEdge> Edges { get; }

        long VertexCount { get; }

        long EdgeCount { get; }

        bool AddVertex(IVertex theVertex);

        bool RemoveVertex(IVertex theVertex);

        bool AddEdge(IEdge theEdge);

        bool RemoveEdge(IEdge theEdge);

        void ClearVertices();

        void ClearEdges();
    }
}
