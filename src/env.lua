-- This code implements Lover constants in your game
-- Learn more at https://github.com/Wolfyxon/lover/wiki/Constants 

os = os or {}

local originalGetEnv = os.getenv or function () end
local loverConsts = {}

function os.getenv(varname)
    return loverConsts[varname] or originalGetEnv(varname)
end

