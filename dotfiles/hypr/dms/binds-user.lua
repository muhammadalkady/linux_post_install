-- Optional per-user keybind overrides (managed by DMS). Loaded after default binds.

-- Frequently used applications
hl.bind("SUPER + E", hl.dsp.exec_cmd("nautilus --new-window"), { description = "Open Nautilus" })
hl.bind("SUPER + B", hl.dsp.exec_cmd("google-chrome"), { description = "Open Google Chrome" })
hl.bind("SUPER + A", hl.dsp.exec_cmd("~/.local/share/JetBrains/Toolbox/scripts/studio"), { description = "Open Android Studio" })
hl.bind("CTRL + R", hl.dsp.exec_cmd("~/.local/share/JetBrains/Toolbox/scripts/rustrover"), { description = "Open RustRover" })
hl.bind("SUPER + Z", hl.dsp.exec_cmd("zed"), { description = "Open Zed" })

-- Application menu (SUPER + SPACE remains available as the DMS default)
hl.bind("SUPER + D", hl.dsp.exec_cmd("dms ipc call spotlight toggle"), { description = "Open application menu" })

-- Move windows between workspaces without following them to the destination.
local function bind_silent_workspace_move(keys, workspace)
    hl.unbind(keys)
    hl.bind(
        keys,
        hl.dsp.window.move({ workspace = workspace, follow = false }),
        { description = "Move window silently to workspace " .. workspace }
    )
end

for workspace = 1, 9 do
    bind_silent_workspace_move("SUPER + SHIFT + " .. workspace, tostring(workspace))
end

bind_silent_workspace_move("SUPER + CTRL + down", "e+1")
bind_silent_workspace_move("SUPER + CTRL + up", "e-1")
bind_silent_workspace_move("SUPER + CTRL + U", "e+1")
bind_silent_workspace_move("SUPER + CTRL + I", "e-1")
bind_silent_workspace_move("SUPER + SHIFT + Page_Down", "e+1")
bind_silent_workspace_move("SUPER + SHIFT + Page_Up", "e-1")
bind_silent_workspace_move("SUPER + SHIFT + U", "e+1")
bind_silent_workspace_move("SUPER + SHIFT + I", "e-1")
bind_silent_workspace_move("SUPER + CTRL + mouse_down", "e+1")
bind_silent_workspace_move("SUPER + CTRL + mouse_up", "e-1")
