{
  config,
  pkgs,
  lib,
  self,
  ...
}:
let
  cfgProgram = config.programs.postar;
  cfgService = config.service.postar;
  postarPackage = self.packages.${pkgs.system}.default;
  tomlFormat = pkgs.formats.toml { };
in
with lib;
{
  options.programs.postar = {
    enable = mkEnableOption "postar";
    package = mkOption {
      type = types.package;
      default = postarPackage;
      description = "The postar package to use.";
    };
    config = {
      type = tomlFormat.type;
      description = ''
        Configuration written to {file}`$XDG_CONFIG_HOME/postar/config.toml`
        See <https://github.com/filiptrplan/postar?tab=readme-ov-file#toml-configuration-reference> for details.
      '';
      default = { };
      example = ''
        imap = [
          {
            name = "Main";
            server = "mail.example.com";
            port = 3993;
            username = "user@example.com";
            password = "pass";
            default = true;
          }
          {
            name = "Secondary";
            server = "mail.example.org";
            port = 3993;
            self_signed_cert = true;
            username = "user2@example.org";
            password = "pass";
          }
        ];
        postar = {
          polling_delay = 5;
        }
      '';
    };
    rules = {
      type = types.str;
      default = "";
      description = ''
        Rules file written to {file}`$XDG_CONFIG_HOME/postar/rules.ptar`
        See <https://github.com/filiptrplan/postar?tab=readme-ov-file#rule-dsl> for details.
      '';
      example = ''
        folder test1 { name: "INBOX.tests1" }
        folder inbox { name: "INBOX" }

        rule test {
          matcher: subject contains "TEST"
          action: moveto [test1]
        }


        rule complex_rule {
          matcher: and [
            subject contains "HI"
            or [
              to startswith "MyName"
              not from contains "TEST"
            ]
          ]
          action: moveto [test1]
        }
      '';
    };
  };
  options.services.postar = {
    enable = mkEnableOption "postar service";
    package = mkOption {
      type = types.package;
      default = postarPackage;
      description = "The postar package to use.";
    };
  };
  config =
    mkIf cfgProgram.enable {
      home.packages = mkIf (cfgProgram.package != null) [ cfgProgram.package ];
      xdg.configFile."postar/config.toml" = mkIf (cfgProgram.config != { }) {
        source = tomlFormat.generate "config.toml" cfgProgram.config;
      };
      xdg.configFile."postar/rules.ptar" = mkIf (cfgProgram.rules != "") {
        text = cfgProgram.rules;
      };
    }
    // mkIf cfgService.enable {
      systemd.user.services.postar = {
        Unit = {
          Description = "An email filtering service.";
          After = [ "network-online.target" ];
        };
        Install = {
          WantedBy = [ "default.target" ];
          Restart = "on-failure";
        };
        Service = {
          ExecStart = "${cfgService.package}/bin/postar";
        };
      };
    };
}
