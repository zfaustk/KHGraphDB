using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    public abstract class Algorithm : IAlgorithm
    {
        public abstract void BeginAlgorithm(IGraph theGraph);

        public abstract void EndAlgorithm(IGraph theGraph);
    }
}
