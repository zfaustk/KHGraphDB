namespace KHGraphDB.Structure.Interface
{
    public interface IEdge : IDBObject
    {
        IGraph Graph { get; set; }

        IVertex Source { get; }

        IVertex Target { get; }

        IType Type { get; set; }
    }
}
