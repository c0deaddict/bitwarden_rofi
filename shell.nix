with import <nixpkgs> {};

mkShell {
  buildInputs = [ dbus_libs ];
}
