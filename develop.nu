#!/usr/bin/env nu

def main []  {
  print "Fuzz targets:"
  print "cargo fuzz run colour_to_raw -- -max_total_time=60"
  print "cargo fuzz run push_unchecked -- -max_total_time=60"
  nix develop -c $env.SHELL
}
