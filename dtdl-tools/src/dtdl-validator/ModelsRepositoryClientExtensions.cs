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
    /// It asynchronously gets the model definitions (specified in DTDL) associated with each of the provided DTMIs from the models repository.
    /// </summary>
    /// <param name="modelRepoClient">The models repository client.</param>
    /// <param name="dtmis">The DTMIs whose model definitions should be returned.</param>
    /// <param name="cancellationToken">The cancellation token.</param>
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