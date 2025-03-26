-- This code implements Lover constants in your game
-- Learn more at https://github.com/Wolfyxon/lover/wiki/Constants 

os = os or {}
os._getenv = os.getenv or function() end

local loverConsts = {}

function os.getenv(varname, noLover)
    if noLover then
        return os._getenv(varname)
    end

    return loverConsts[varname] or os._getenv(varname)
end

