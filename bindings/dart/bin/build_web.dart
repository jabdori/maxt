import 'dart:io';
import 'dart:isolate';

const _usage = 'Usage: dart run maxt:build_web [--release]';

Future<void> main(List<String> args) async {
  if (args.any((argument) => argument != '--release')) {
    stdout.writeln(_usage);
    exitCode = args.contains('--help') ? 0 : 64;
    return;
  }
  if (!File('pubspec.yaml').existsSync() || !Directory('web').existsSync()) {
    stderr.writeln('Run this command from the Dart or Flutter web app root.');
    exitCode = 64;
    return;
  }

  final library = await Isolate.resolvePackageUri(
    Uri.parse('package:maxt/maxt.dart'),
  );
  if (library == null || library.scheme != 'file') {
    stderr.writeln('Could not locate the installed maxt package.');
    exitCode = 70;
    return;
  }

  final packageRoot = File.fromUri(library).parent.parent;
  final command = <String>[
    'run',
    'flutter_rust_bridge:flutter_rust_bridge',
    'build-web',
    '--dart-root',
    packageRoot.path,
    '--rust-root',
    packageRoot.uri.resolve('rust/').toFilePath(),
    '--output',
    Directory('web').absolute.path,
    if (args.contains('--release')) '--release',
  ];
  final process = await Process.start(
    Platform.resolvedExecutable,
    command,
    mode: ProcessStartMode.inheritStdio,
  );
  exitCode = await process.exitCode;
}
