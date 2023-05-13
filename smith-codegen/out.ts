
export class Optional<T>{
   private tag: 'Some' | 'None'
   private val?: T
   private constructor(tag: 'Some' | 'None', val?: T){
     this.tag = tag;
     this.val = val;
   }
   
   static Some<T>(v: T){return new Optional("Some",v)}
   as_Some(){
      if (this.tag != 'Some'){
         throw new Error("Enum Optional: trying to cast variant '" + this.tag + "' into 'Some'")
      }
      return this.val as T    
   }

   static None(){return new Optional("None",)}
   as_None(){
      if (this.tag != 'None'){
         throw new Error("Enum Optional: trying to cast variant '" + this.tag + "' into 'None'")
      }
          
   }
   getTag(){ return this.tag }
}

export class PacketType{
   private tag: 'Ack' | 'LogOut' | 'Order'
   private val?: Order<OrderItem>
   private constructor(tag: 'Ack' | 'LogOut' | 'Order', val?: Order<OrderItem>){
     this.tag = tag;
     this.val = val;
   }
   
   static Ack(){return new PacketType("Ack",)}
   as_Ack(){
      if (this.tag != 'Ack'){
         throw new Error("Enum PacketType: trying to cast variant '" + this.tag + "' into 'Ack'")
      }
          
   }

   static LogOut(){return new PacketType("LogOut",)}
   as_LogOut(){
      if (this.tag != 'LogOut'){
         throw new Error("Enum PacketType: trying to cast variant '" + this.tag + "' into 'LogOut'")
      }
          
   }

   static Order(v: Order<OrderItem>){return new PacketType("Order",v)}
   as_Order(){
      if (this.tag != 'Order'){
         throw new Error("Enum PacketType: trying to cast variant '" + this.tag + "' into 'Order'")
      }
      return this.val as Order<OrderItem>    
   }
   getTag(){ return this.tag }
}

export interface OrderItem{
  id: number
  amount: number
}

export interface Order<T>{
  table_number: number
  items: Optional<T[]>
}

export interface Person{
  name: string
  age: number
  desc: Optional<string>
}

export interface Packet{
  id: number
  payload: PacketType
}
