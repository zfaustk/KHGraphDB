using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    public interface IAlgorithm
    {
        void BeginAlgorithm(IGraph theGraph);

        void EndAlgorithm(IGraph theGraph);
    }
}
