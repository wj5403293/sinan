-- maps range_type 已默认给出

local mode = (range_type == "All") or (range_type ~= "Default")

local result = {}
local proc_name = env.name() -- 获取当前选定进程名称

