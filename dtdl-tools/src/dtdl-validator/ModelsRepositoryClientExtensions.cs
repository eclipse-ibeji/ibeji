
using Azure.IoT.ModelsRepository;
using DTDLParser;
using System.Runtime.CompilerServices;

internal static class ModelsRepositoryClientExtensions
{
    public static async IAsyncEnumerable<string> ParserDtmiResolverAsync(
        this ModelsRepositoryClient client, IReadOnlyCollection<Dtmi> dtmis,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        Console.WriteLine("HERE");
        foreach (var dtmi in dtmis.Select(s => s.AbsoluteUri))
        {
            Console.WriteLine($"DTMI >>> {dtmi}");
            var result = await client.GetModelAsync(dtmi, ModelDependencyResolution.Disabled, ct);
            Console.WriteLine("DTMI <<<");
            yield return result.Content[dtmi];
        }
    }
}