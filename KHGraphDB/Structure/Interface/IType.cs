using System.Collections.Generic;

namespace KHGraphDB.Structure.Interface
{
    public interface IType : IDBObject
    {
        IGraph Graph { get; set; }

        string Name { get; set; }

        IEnumerable<IVertex> Vertices { get; }

        long VertexCount { get; }

        bool AddVertex(IVertex theVertex);

        bool RemoveVertex(IVertex theVertex);

        void ClearVertices();
    }
}
