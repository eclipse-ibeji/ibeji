// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using DTDLParser;
using DTDLParser.Models;
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

        var parser = new ModelParser();

        bool failureOccured = false;
        foreach (var file in files)
        {
            try
            {
                var dtdl = File.ReadAllText(file);
                parser.ParseAsync(dtdl).GetAwaiter().GetResult();
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
