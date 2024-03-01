// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using Azure.IoT.ModelsRepository;
using DTDLParser;
using System.Runtime.CompilerServices;

internal static class ModelsRepositoryClientExtensions
{
    /// <summary>
    /// The Parser's DTMI resolver.
    /// It asynchronously gets from the models repositpory the DTDL content associated witn each of the provided DTMIs.
    /// </summary>
    /// <param name="modelRepoClient">The models repository client.</param>
    /// <param name="dtmis"></param>
    /// <param name="cancellationToken">The cancellation topken.</param>
    /// <returns>The model definitions for the provided DTMIs.</returns>
    public static async IAsyncEnumerable<string> ParserDtmiResolverAsync(
        this ModelsRepositoryClient modelRepoClient, IReadOnlyCollection<Dtmi> dtmis,
        [EnumeratorCancellation] CancellationToken cancellationToken = default)
    {
        foreach (var dtmi in dtmis.Select(s => s.AbsoluteUri))
        {
            var result = await modelRepoClient.GetModelAsync(dtmi, ModelDependencyResolution.Disabled, cancellationToken);
            yield return result.Content[dtmi];
        }
    }
}