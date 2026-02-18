{ self, ... }:
{
  flake.homeModules.weathr =
    {
      config,
      lib,
      pkgs,
      ...
    }:

    let
      cfg = config.programs.weathr;
      tomlFormat = pkgs.formats.toml { };

      configDir = if pkgs.stdenv.isDarwin then "Library/Application Support/weathr" else ".config/weathr";

    in
    {
      options.programs.weathr = {
        enable = lib.mkEnableOption "weathr";

        package = lib.mkOption {
          type = lib.types.package;
          default = self.packages.${pkgs.system}.default;
          description = "The weathr package to install.";
        };

        settings = lib.mkOption {
          type = tomlFormat.type;
          default = { };
          description = ''
            Configuration written to the application's config directory.

            See <https://github.com/veirt/weathr#configuration> for available options.
          '';
          example = lib.literalExpression ''
            {
              hide_hud = false;
              silent = false;
              location = {
                latitude = 40.7128;
                longitude = -74.0060;
                auto = false;
                hide = false;
              };
              units = {
                temperature = "celsius";
                wind_speed = "kmh";
                precipitation = "mm";
              };
            }
          '';
        };
      };

      config = lib.mkIf cfg.enable {
        home.packages = [ cfg.package ];

        home.file."${configDir}/config.toml" = lib.mkIf (cfg.settings != { }) {
          source = tomlFormat.generate "weathr-config" cfg.settings;
        };
      };
    };
}
