namespace    = "alaydriem"
name         = "quirks"
entry        = "alaydriem_quirks.dll"
api_version  = 2
amml_version = "26.60.6"
entries = [
  "ClientInstance.update",
  "ClientInstance.getMinecraftInput",
  "OreUIRouter.pushRoute",
  "OreUIRouter.replaceRoute",
]

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
