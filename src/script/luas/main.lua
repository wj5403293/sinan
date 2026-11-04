
-- 1. 获取目标模块基址
local game_mod = mu.getModuleByName("module")
if not game_mod then return print("错误: 找不到 module") end

-- 2. 计算结构体指针地址 (模块基址 + 一级静态偏移)
local ptr_address = game_mod.base + 0x1AB200

-- 3. 安全读取一级指针
local success, player_base = pcall(function() return mv.u64[ptr_address] end)
if not success or player_base == 0 then return print("无法读取玩家指针") end

-- 4. 读取并修改二级属性 (假定血量在偏移 0x40，速度在 0x48)
local hp_addr = player_base + 0x40
local speed_addr = player_base + 0x48

print("当前血量:", mv.i32[hp_addr])
mv.i32[hp_addr] = 10000          -- 修改血量
mv.f32[speed_addr] = 250.0       -- 修改速度
print("数据修改完成！")