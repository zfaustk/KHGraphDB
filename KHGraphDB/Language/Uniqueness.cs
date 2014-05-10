namespace KHGraphDB.Language
{
    /// <summary>
    /// Neo4j traversal uniqueness. NODE_GLOBAL is the MATCH default:
    /// a vertex appears at most once on a path.
    /// </summary>
    public enum Uniqueness
    {
        NodePath,
        NodeGlobal,
        RelationshipPath
    }
}
