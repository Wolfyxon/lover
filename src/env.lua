os = os or {}

local originalGetEnv = os.getenv or function () end
local consts = {}

function os.getenv(varname)
    return consts[varname] or originalGetEnv(varname)
end

