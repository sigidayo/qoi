#!/usr/bin/env nu

def main []  {
  print "Run tests with:   cargo test"
  print "Fuzz with:        cargo fuzz run colour_to_raw -j 4 -- -max_total_time=60"
  nix develop -c $env.SHELL
}
