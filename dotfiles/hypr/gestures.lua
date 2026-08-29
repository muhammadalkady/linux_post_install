-------------------------------------------------------
-- Gestures
-------------------------------------------------------

-- Workspaces
hl.gesture({
    fingers = 3,
    direction = "vertical",
    action = "workspace"
})

-- Fullscreen on  
hl.gesture({ fingers = 4, direction = "pinchout", action = function ()
    hl.dispatch(hl.dsp.window.fullscreen({ action="set" })) 
end})

-- Fullscreen off  
hl.gesture({ fingers = 4, direction = "pinchin", action = function ()
    hl.dispatch(hl.dsp.window.fullscreen({ action="unset" })) 
end})
