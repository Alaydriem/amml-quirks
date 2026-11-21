namespace    = "alaydriem"
name         = "quirks"
entry        = "alaydriem_quirks.dll"
api_version  = 3

uses {
  feature "idle" {
    event "client.tick" {
      version  = 1
      required = true
    }
    event "mods.loaded" {
      version  = 1
      required = true
    }
    event "registry.provider_ready" {
      version  = 1
      required = true
    }
    event "registry.provider_gone" {
      version  = 1
      required = true
    }
    event "input.lease_suspended" {
      version  = 1
      required = false
    }
    event "input.lease_resumed" {
      version  = 1
      required = false
    }
    event "input.lease_lost" {
      version  = 1
      required = false
    }
    event "input.injected_released" {
      version  = 2
      required = false
    }
    event "overlay.screenshot_taken" {
      version  = 1
      required = false
    }
    event "overlay.screenshot_written" {
      version  = 1
      required = false
    }
    area "input.lease" {
      version  = 1
      required = false
    }
  }
  feature "idle_screen" {
    event "client.idle_screen_open" {
      version  = 1
      required = true
    }
  }
  feature "keep_alive" {
    area "client.activity" {
      version  = 1
      required = true
    }
  }
}

listing {
  title   = "Quirks"
  summary = "Small fixes for things in Minecraft that annoy players."
  authors = ["Charles R. Portwood II"]
  license = "BSD-3-Clause"
  source  = "https://github.com/alaydriem/amml-quirks"
  homepage = "https://github.com/alaydriem/amml-quirks"
  tags    = ["idle", "afk", "client", "utility"]
  icon    = "icon.png"
}
