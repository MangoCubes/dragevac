{
  config,
  inputs,
  pkgs,
  lib,
  ...
}:
{
  options = {
    programs.dragevac = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Enable DragEvac, a drag and drop manager.";
      };
      settings = lib.mkOption {
        type = lib.types.attrs;
        default = { };
        description = "Any settings that are not yet implemented in the flake goes here. They will automatically be converted into JSON.";
      };
    };
  };
  config =
    let
      cfg = config.programs.dragevac;
    in
    lib.mkIf cfg.enable {
      # Install package
      home.packages = [
        inputs.dragevac.packages."${pkgs.stdenv.hostPlatform.system}".default
      ];
      xdg = {
        # Create config.json
        configFile."dragevac/config.json".text = (builtins.toJSON (cfg.settings));
      };
    };
}
