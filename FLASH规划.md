共2m flash，前1m：boot+data 后1m：app双区各512kb。

基地址:0x0800_0000
boot 前128kb
剩余data 或者为空 0x08002_0000 偏移0x2_0000

app1:0x810_0000 偏移10_0000 扇区12开始 msg + app 前1kb固件信息
app2:0x818_0000 偏移18_0000 扇区20开始 msg + app 前1kb固件信息