-- This code implements Lover constants in your game
-- Learn more at https://github.com/Wolfyxon/lover/wiki/Constants 

os = os or {}

local originalGetEnv = os.getenv or function () end
local consts = {}

function os.getenv(varname)
    return consts[varname] or originalGetEnv(varname)
end

