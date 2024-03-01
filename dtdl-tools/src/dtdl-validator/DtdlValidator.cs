// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using DTDLParser;
using DTDLParser.Models;
using Azure;
using Azure.IoT.ModelsRepository;
using System;
using System.Collections.Generic;
using System.CommandLine;
using System.CommandLine.NamingConventionBinder;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

/// <summary>
/// The DTDL Validator app.
/// </summary>
class Program
{
    // Exit codes.
    private const int EXIT_SUCCESS = 0;
    private const int EXIT_FAILURE = 1;

    /// <summary>
    /// Convert a DTDL file's path to a Digital Twin Model Identifier (DTMI).
    /// </summary>
    /// <param name="dtdlFilePath">The DTDL file's full path.</param>
    /// <param name="dtdlDirPath">The DTDL directory's path.</param>
    /// <param name="extension">The extension used by the DTDL files.</param>
    /// <returns>The corresponding DTMI.</returns>
    static string ConvertToDTMI(string dtdlFilePath, string dtdlDirPath, string extension)
    {
        // Strip off the directory path and the extension.
        string dtmiPath = dtdlFilePath.Substring(dtdlDirPath.Length + 1, dtdlFilePath.Length - dtdlDirPath.Length - extension.Length - 2);
        // Replace each directory separator with a colon and the hyphen with a semicolon.
        string dtmi = dtmiPath.Replace(Path.DirectorySeparatorChar, ':').Replace('-', ';');
        return dtmi;
    }

    /// <summary>
    /// This method validates all of the DTDL files with the provided extension that are located
    /// under the provided directory.
    /// </summary>
    /// <param name="dtdlDirectory">The directory that contains the DTDL files that we wish to validate.</param>
    /// <param name="extension">The extension used by the DTDL files.</param>
    /// <returns>
    /// EXIT_SUCCESS when all if the DTDL files are valid.
    /// EXIT_FAILURE when any of the DTDL files are NOT valid.
    /// </returns>
    static int ValidateDtdl(DirectoryInfo dtdlDirectory, String extension)
    {
        if (!dtdlDirectory.Exists)
        {
            Console.WriteLine($"Directory {dtdlDirectory.FullName} does not exist.");
            return EXIT_FAILURE;
        }

        var files = Directory.GetFiles(dtdlDirectory.FullName, $"*.{extension}", SearchOption.AllDirectories);

        if (!files.Any())
        {
            Console.WriteLine($"No files with extension .{extension} found in directory {dtdlDirectory}");
            return EXIT_FAILURE;
        }

        var modelRepoClient = new ModelsRepositoryClient(new Uri(dtdlDirectory.FullName));

        var parser = new ModelParser(new ParsingOptions()
        {
            DtmiResolverAsync = modelRepoClient.ParserDtmiResolverAsync
        });

        bool failureOccured = false;
        foreach (var file in files)
        {
            try
            {
                string dtmi = ConvertToDTMI(file, dtdlDirectory.FullName, extension);
                var model = modelRepoClient.GetModelAsync(dtmi).GetAwaiter().GetResult();
                var modelDictionary = parser.ParseAsync(model.Content[dtmi]).GetAwaiter().GetResult();
                Console.WriteLine($"{file} - ok");
            }
            catch (ParsingException ex)
            {
                Console.WriteLine($"{file} - failed");
                foreach (var error in ex.Errors) {
                    Console.WriteLine($"  {error}");
                }
                failureOccured = true;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"{file} - failed");
                Console.WriteLine($"  {ex.ToString()}");
                failureOccured = true;
            }
        }

        if (failureOccured)
        {
            return EXIT_FAILURE;
        }
        else
        {
            return EXIT_SUCCESS;
        }
    }

    static async Task<int> Main(string[] args)
    {
        var directoryArgument =
            new Argument<DirectoryInfo>(
                "directory",
                description: "The directory that contains the DTDL files.");
        var extensionOption =
            new Option<string>(
                "-e",
                getDefaultValue: () => "json",
                description: "The file extension used by the DTDL files.");

        int exitCode = EXIT_SUCCESS;

        var cmd = new RootCommand();
        cmd.AddArgument(directoryArgument);
        cmd.AddOption(extensionOption);
        cmd.SetHandler((directory, extension) =>
            {
                exitCode = ValidateDtdl(directory!, extension!);
            },
            directoryArgument, extensionOption);

        await cmd.InvokeAsync(args);

        return exitCode;
    }
}
