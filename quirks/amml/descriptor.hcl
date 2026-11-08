namespace    = "alaydriem"
name         = "quirks"
display_name = "Quirks"
version      = "26.62.0"
author       = "Alaydriem"
config_path  = "config.hcl"

section "idle" {
  title = "Idle"

  field "idle.enabled" {
    label   = "Stop idle timeout"
    help    = "Stops the away screen and the low frame rate after 15 minutes with no input. Does not move your player"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
}
