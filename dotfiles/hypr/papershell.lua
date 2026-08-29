-- PaperShell-style compositor grain. Reloading the config starts enabled.
local shader_dir = os.getenv("HOME") .. "/.config/hypr/shaders/"
local modes = {
	{ name = "Paper grain", shader = shader_dir .. "papershell.frag" },
	{ name = "Book Comfort", shader = shader_dir .. "book-comfort.frag" },
	{ name = "Off", shader = "" },
}
local mode = 1

-- Hyprland 0.55/0.56 can leave native Wayland clients such as Chrome with
-- stale partial buffers when full damage tracking is used on a fractionally
-- scaled output. Monitor-level tracking avoids the flicker while remaining
-- damage-driven when the output is idle.
hl.config({
	debug = {
		damage_tracking = 1,
	},
})

local function apply_papershell()
	hl.config({
		decoration = {
			screen_shader = modes[mode].shader,
		},
	})
end

apply_papershell()

hl.bind("SUPER + SHIFT + G", function()
	mode = mode % #modes + 1
	apply_papershell()
	-- The shader swap is applied on the next rendered frame. Delay the full
	-- redraw until after that frame so every output is refreshed with the new
	-- shader instead of waiting for pointer damage.
	hl.timer(function()
		hl.dispatch(hl.dsp.force_renderer_reload())
	end, { timeout = 50, type = "oneshot" })
end, { description = "Cycle paper display modes" })
