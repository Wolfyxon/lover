local originalGetEnv = os.getenv
local consts = {}

function os.getenv(varname)
    return consts[varname] or originalGetEnv(varname)
end
