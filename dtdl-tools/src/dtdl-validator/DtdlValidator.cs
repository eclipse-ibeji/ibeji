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

class Program
{
    // Exit codes.
    private const int EXIT_SUCCESS = 0;
    private const int EXIT_FAILURE = 1;

    static string ConvertToDTMI(string filepath, string dirpath, string extension)
    {
        string relativepath = filepath.Substring(dirpath.Length + 1, filepath.Length - dirpath.Length - extension.Length - 2);
        string dtmi = relativepath.Replace('/', ':').Replace('-', ';'); // .ToLower();
        return dtmi;
    }

    /// <summary>
    /// This method validates all of the DTDL files with the provided extension that are located
    /// under the provided directory.
    /// </summary>
    /// <param name="directory">The directory that contains the DTDL files that we wish to validate.</param>
    /// <param name="extension">The extension used by the DTDL files.</param>
    /// <returns>
    /// EXIT_SUCCESS when all if the DTDL files are valid.
    /// EXIT_FAILURE when any of the DTDL files are NOT valid.
    /// </returns>
    static int ValidateDtdl(DirectoryInfo directory, String extension)
    {
        if (!directory.Exists)
        {
            Console.WriteLine($"Directory {directory.FullName} does not exist.");
            return EXIT_FAILURE;
        }

        var files = Directory.GetFiles(directory.FullName, $"*.{extension}", SearchOption.AllDirectories);

        if (!files.Any())
        {
            Console.WriteLine($"No files with extension .{extension} found in directory {directory}");
            return EXIT_FAILURE;
        }

        var dmrClient = new ModelsRepositoryClient(new Uri(directory.FullName));

        var parser = new ModelParser(new ParsingOptions()
        {
            DtmiResolverAsync = dmrClient.ParserDtmiResolverAsync
        });

        bool failureOccured = false;
        foreach (var file in files)
        {
            try
            {                
                string dtmi = ConvertToDTMI(file, directory.FullName, extension);
                Console.WriteLine("DTMI = {0}", dtmi);

                // var dtdl = File.ReadAllText(file);
                // parser.ParseAsync(dtdl).GetAwaiter().GetResult();
                var model = dmrClient.GetModelAsync(dtmi).GetAwaiter().GetResult();
                Console.WriteLine("For {0} the model cardinality is {1}", dtmi, model.Content.Count);
                // Console.WriteLine("Content = {0}", model.Content[dtmi]);
                var dictParsed = parser.ParseAsync(model.Content[dtmi]).GetAwaiter().GetResult();                
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
            catch (ResolutionException ex)
            {
                Console.WriteLine($"{file} - failed");
                Console.WriteLine($"  {ex.ToString()}");
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
