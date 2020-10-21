using System.Runtime.InteropServices;
using System.Text;
using Unity.Collections;
using UnityEngine;

public class ObjParserTest : MonoBehaviour
{
    public Material defaultMaterial;
    public string importPath;

    public unsafe static class ExternalDll
    {
        /// <summary>
        /// Parses an obj file from file path
        /// </summary>
        /// <param name="filePathBytes">UTF-8 bytes of the path</param>
        /// <param name="filePathByteCount">The byte count of filePathBytes</param>
        /// <returns>A pointer to the handle. If null, then an error occured while parsing.</returns>
        [DllImport("objparsers.dll", EntryPoint = "parse_obj_from_file_path", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Auto)]
        public static extern void* ParseObjFromFilePath(byte* filePathBytes, uint filePathByteCount);

        /// <summary>
        /// Parses an obj file from bytes
        /// </summary>
        /// <param name="fileContentsBytes">Bytes of the file's content</param>
        /// <param name="fileContentsByteCount">The byte count of fileContentsBytes</param>
        /// <returns>A pointer to the handle. If null, then an error occured while parsing.</returns>
        [DllImport("objparsers.dll", EntryPoint = "parse_obj", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Auto)]
        public static extern void* ParseObj(byte* fileContentsBytes, uint fileContentsByteCount);

        /// <summary>
        /// Note: the count is the count of the vertices, which is equal to array size / 3
        /// </summary>
        [DllImport("objparsers.dll", EntryPoint = "get_vertex_count", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Auto)]
        public static extern uint GetVertexCount(void* handle);

        /// <summary>
        /// 3x 4-byte floats per vertex
        /// </summary>
        [DllImport("objparsers.dll", EntryPoint = "get_vertex_positions", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Auto)]
        public static extern float* GetVertexPositions(void* handle);

        /// <summary>
        /// Index count is equal to the array size (or triangle count * 3)
        /// </summary>
        [DllImport("objparsers.dll", EntryPoint = "get_index_count", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Auto)]
        public static extern uint GetIndexCount(void* handle);

        /// <summary>
        /// 3x 4-byte unsigned integers per triangle (1x 4 byte per index)
        /// </summary>
        [DllImport("objparsers.dll", EntryPoint = "get_indices", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Auto)]
        public static extern uint* GetIndices(void* handle);

        /// <summary>
        /// Destroys the handle and frees all resources
        /// </summary>
        /// <param name="handle"></param>
        [DllImport("objparsers.dll", EntryPoint = "destroy_handle", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Auto)]
        public static extern void DestroyHandle(void* handle);
    }

    private void Start()
    {
        unsafe
        {
            void* handle = null;
            byte[] pathBytes = Encoding.UTF8.GetBytes(importPath);
            System.Diagnostics.Stopwatch sw = new System.Diagnostics.Stopwatch();
            long totalMs = 0;

            fixed (byte* ptr = pathBytes)
            {
                sw.Restart();
                handle = ExternalDll.ParseObj(ptr, (uint)pathBytes.Length);
                sw.Stop();
                Debug.Log("parsed in " + sw.ElapsedMilliseconds + "ms");
                totalMs += sw.ElapsedMilliseconds;
            }

            try
            {
                sw.Restart();
                uint vertexCount = ExternalDll.GetVertexCount(handle);
                float* vertices = ExternalDll.GetVertexPositions(handle);

                uint indexCount = ExternalDll.GetIndexCount(handle);
                uint* indices = ExternalDll.GetIndices(handle);
                sw.Stop();
                Debug.Log("mesh data retrieved in " + sw.ElapsedMilliseconds + "ms");
                totalMs += sw.ElapsedMilliseconds;

                if (true)
                {
                    sw.Restart();
                    GameObject go = new GameObject();
                    go.AddComponent<MeshRenderer>().material = defaultMaterial;

                    Mesh mesh = new Mesh();
                    go.AddComponent<MeshFilter>().sharedMesh = mesh;
                    mesh.indexFormat = vertexCount > ushort.MaxValue ? UnityEngine.Rendering.IndexFormat.UInt32 : UnityEngine.Rendering.IndexFormat.UInt16;

                    NativeArray<Vector3> verticesNative = Unity.Collections.LowLevel.Unsafe.NativeArrayUnsafeUtility
                        .ConvertExistingDataToNativeArray<Vector3>(vertices, (int)vertexCount, Allocator.None);
#if ENABLE_UNITY_COLLECTIONS_CHECKS
                    Unity.Collections.LowLevel.Unsafe.NativeArrayUnsafeUtility.SetAtomicSafetyHandle(
                        ref verticesNative, Unity.Collections.LowLevel.Unsafe.AtomicSafetyHandle.Create());
#endif

                    NativeArray<int> indicesNative = Unity.Collections.LowLevel.Unsafe.NativeArrayUnsafeUtility
                        .ConvertExistingDataToNativeArray<int>(indices, (int)indexCount, Allocator.None);
#if ENABLE_UNITY_COLLECTIONS_CHECKS
                    Unity.Collections.LowLevel.Unsafe.NativeArrayUnsafeUtility.SetAtomicSafetyHandle(
                        ref indicesNative, Unity.Collections.LowLevel.Unsafe.AtomicSafetyHandle.Create());
#endif

                    mesh.SetVertices(verticesNative);
                    mesh.SetIndices(indicesNative, MeshTopology.Triangles, 0);

                    sw.Stop();
                    Debug.Log("gameobject created in " + sw.ElapsedMilliseconds + "ms");
                    totalMs += sw.ElapsedMilliseconds;

                    sw.Restart();
                    mesh.RecalculateNormals();
                    mesh.RecalculateBounds();
                    sw.Stop();
                    Debug.Log("normals and bounds recalculated in " + sw.ElapsedMilliseconds + "ms");
                    totalMs += sw.ElapsedMilliseconds;

                    Debug.Log("total: " + totalMs + "ms");
                }
            }
            finally
            {
                ExternalDll.DestroyHandle(handle);
            }
        }
    }
}
