local originalGetEnv = os.getenv
local loverConstants = {}

function os.getenv(varname)
    return loverConstants[varname] or originalGetEnv(varname)
end