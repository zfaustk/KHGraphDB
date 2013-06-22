using System.Collections.Generic;

namespace KHGraphDB.Structure.Interface
{
    public interface IVertex : IDBObject
    {
        IGraph Graph { get; set; }

        IType Type { get; set; }

        IEnumerable<IEdge> OutgoingEdges { get; }

        IEnumerable<IEdge> IncomingEdges { get; }

        long Degree { get; }

        long InDegree { get; }

        long OutDegree { get; }

        bool AddOutgoingEdge(IEdge theEdge);

        bool AddIncomingEdge(IEdge theEdge);

        bool RemoveOutgoingEdge(IEdge theEdge);

        bool RemoveIncomingEdge(IEdge theEdge);
    }
}
