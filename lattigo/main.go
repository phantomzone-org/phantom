package main

import(
	"fmt"
	"github.com/Pro7ech/lattigo/ring"
)

// LUI: write constant to register
// AUIPC: add constant to counter and stores the constant to register
// JAL: add constant to counter and stores counter +1 in register
// B--: compare two register and update counter based on comparison result, according to the stored constant


func main(){
	r, err := ring.NewRing(64, 65537, 1)

	if err != nil{
		panic(err)
	}

	r.GenNTTTable()

	register := NewRegister(r, [32]uint64{
		2, 1, 0, 0, 0, 0, 0, 0, 
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0})

	counter := NewCounter(r)

	instruction := NewInstructions(r)

	// Update counter Instructions
	instruction.CounterUpdate[0] = 1
	for i := range 8{
		instruction.CounterUpdate[r.N-i-1] = uint64(r.Modulus-1)
	}

	// Operationns Instructions
	instruction.Operation[0] = 0
	for i := range 8{
		instruction.Operation[r.N-i-1] = r.Modulus-uint64(2*r.N-1)
	}

	// OperatOperationns inputs 0
	instruction.RS1[0] = 0
	for i := range 8{
		instruction.RS1[r.N-i-1] = 0
	}

	// OperatOperationns inputs 1
	instruction.RS2[0] = uint64(2*r.N-1)
	for i := range 8{
		instruction.RS2[r.N-i-1] = r.Modulus-uint64(2*r.N-1)
	}

	fmt.Println(instruction.RS2)

	// OperatOperationns output
	instruction.RDPositive[0] = uint64(2*r.N-1)
	for i := range 8{
		instruction.RDPositive[r.N-i-1] = r.Modulus-uint64(2*r.N-1)
	}

	instruction.RDNegative[0] = 1
	for i := range 8{
		instruction.RDNegative[r.N-i-1] = r.Modulus-1
	}

	// Write instructOperationn
	// add(1, 2) ->
	// mul(1, 2) ->
	instruction.RegisterWriteBoolean[0] = 1
	for i := range 8{
		instruction.RegisterWriteBoolean[r.N-i-1] = r.Modulus-1
	}

	fmt.Println(instruction.CounterUpdate)
	fmt.Println()

	instruction.NTT(r)

	Run(r, Instructions, register, counter)

	r.INTT(register.Poly, register.Poly)

	fmt.Println(register.Poly)

}

var OperationsArithmetic = []func(a, b uint64) (c uint64, counter uint64){
	func(a, b uint64) (c uint64){
		return a + b
	},
	func(a, b uint64) (c uint64){
		return a * b
	},
	
	func(a, b uint64) (c uint64){
		return 0, 0
	},
}

var OperationsUpdatedCounter = []func(a, b, c, counter uint64) (counter uint64){
	func(a, b uint64) (c uint64){
		if a >= b{
			return counter
		}else{
			return counter+c
		}
	},
}


func Run(r *ring.Ring, instruction Instructions, register Register, counter Counter){
	buf := r.NewPoly()
	for i := range 4{
		fmt.Println("i:", i)
		r.INTT(counter.Poly, buf)
		fmt.Println(buf)
		if buf[r.N-1] == 1{
			break
		}
		instruction.Update(r , counter, register)
		fmt.Println()
	}
}

type Counter struct{
	ring.Poly
}

func NewCounter(r *ring.Ring) Counter{
	out := Counter{r.NewPoly()}
	out.Poly[0] = 1
	r.NTT(out.Poly, out.Poly)
	return out
}

type Register struct{
	ring.Poly
}

type Memory struct{
	ring.Poly
}

func NewRegister(r *ring.Ring, values [32]uint64) Register{
	return Register{Pack(r, values[:])}
}

type Instructions struct{
	RS1 ring.Poly // Input 0 ID
	RS2 ring.Poly // Input 1 ID
	RDPositive ring.Poly // Output ID
	RDNegative ring.Poly // Output ID
	Operation ring.Poly // OperatOperationn ID
	CounterUpdate ring.Poly // Counter update
	RegisterWriteBoolean ring.Poly // Write boolean
	LoadFromMemory ring.Poly
}

func NewInstructions(r *ring.Ring) Instructions{
	return Instructions{
		RS1: r.NewPoly(), // Input 0 ID
		RS2: r.NewPoly(), // Input 1 ID
		RDPositive: r.NewPoly(), // Output ID
		RDNegative: r.NewPoly(), // Output ID
		Operation: r.NewPoly(), // OperatOperationn ID
		CounterUpdate: r.NewPoly(), // Counter update
		RegisterWriteBoolean: r.NewPoly(), // Write boolean
		LoadFromMemory: r.NewPoly(),
	}
}

func (ist *Instructions) NTT(r *ring.Ring){
	r.NTT(ist.RS1, ist.RS1)
	r.NTT(ist.RS2, ist.RS2)
	r.NTT(ist.RDPositive, ist.RDPositive)
	r.NTT(ist.RDNegative, ist.RDNegative)
	r.NTT(ist.Operation, ist.Operation)
	r.NTT(ist.CounterUpdate, ist.CounterUpdate)
	r.NTT(ist.RegisterWriteBoolean, ist.RegisterWriteBoolean)
	r.NTT(ist.LoadFromMemory, ist.LoadFromMemory)
	r.NTT(ist.MemoryWriteBoolean, ist.MemoryWriteBoolean)
}

func (ist *Instructions) INTT(r *ring.Ring){
	r.INTT(ist.RS1, ist.RS1)
	r.INTT(ist.RS2, ist.RS2)
	r.INTT(ist.RDPositive, ist.RDPositive)
	r.INTT(ist.RDNegative, ist.RDNegative)
	r.INTT(ist.Operation, ist.Operation)
	r.INTT(ist.CounterUpdate, ist.CounterUpdate)
	r.INTT(ist.RegisterWriteBoolean, ist.RegisterWriteBoolean)
	r.INTT(ist.LoadFromMemory, ist.LoadFromMemory)
	r.INTT(ist.MemoryWriteBoolean, ist.MemoryWriteBoolean)
}


func (ist *Instructions) Update(r *ring.Ring, counter Counter, register Register, memory Memory){

	buf := r.NewPoly()

	// Extract input / output addresses
	RS1 := Read(r, register.Poly, Bootstrap(r, Read(r, ist.RS1, counter.Poly, buf)), buf)
	RS2 := Read(r, register.Poly, Bootstrap(r, Read(r, ist.RS2, counter.Poly, buf)), buf)

	// Evalue all ops
	results := make([]uint64, len(OperatOperationns))
	countersUpdate := make([]uint64, len(OperatOperationns))
	for i := range OperatOperationns{
		results[i], countersUpdate[i] = OperatOperationns[i](RS1, RS2)
	}

	fmt.Println(results)
	fmt.Println(countersUpdate)

	// Load from memory and stores in RD if MemoryWriteBoolean = true
	valueFromMemory := Read(r, register.Poly, Bootstrap(r, Read(r, ist.LoadFromMemory, counter.Poly, buf)), buf)
	storeRegisterFlag := Read(r, ist.RegisterWriteBoolean, counter.Poly, buf)

	// OperatOperationn Selector
	Operation := Bootstrap(r, Read(r, ist.Operation, counter.Poly, buf))

	// Update register
	RDPositive := Bootstrap(r, Read(r, ist.RDPositive, counter.Poly, buf))
	RDNegative := Bootstrap(r, Read(r, ist.RDNegative, counter.Poly, buf))
	Write(r, Read(r, Pack(r, append(results, valueFromMemory)), Operation, buf), storeRegisterFlag, register.Poly, RDPositive, RDNegative)

	// Update counter
	r.MForm(counter.Poly, counter.Poly)
	r.MulCoeffsMontgomery(counter.Poly, Bootstrap(r, Read(r, ist.CounterUpdate, counter.Poly, buf)+Read(r, Operation, Pack(r, countersUpdate), buf)), counter.Poly)

	return
}

// maps i to X^i
func Bootstrap(r *ring.Ring, value uint64) (out ring.Poly){
	out = r.NewPoly()

	sign := (value / uint64(r.N)) & 1
	value %= uint64(r.N)

	if sign == 0{
		out[value] = 1	
	}else{
		out[value] = r.Modulus-1
	}
	r.NTT(out, out)
	return
}

func Read(r *ring.Ring, data, index, buf ring.Poly) uint64{
	r.MForm(data, buf)
	r.MulCoeffsMontgomery(buf, index, buf)
	r.INTT(buf, buf)
	return buf[0]
}

func Write(r *ring.Ring, value uint64, w uint64, data, indexPos, indexNeg ring.Poly){
	r.INTT(data, data)
	fmt.Println("DATA", w, value, data)
	r.NTT(data, data)
	r.MForm(data, data)
	r.MulCoeffsMontgomery(data, indexPos, data)
	r.INTT(data, data)
	data[0] = data[0] * (1-w) + value * w

	r.NTT(data, data)
	r.MForm(data, data)
	r.MulCoeffsMontgomery(data, indexNeg, data)
}

func Pack(r *ring.Ring, values []uint64) (out ring.Poly){
	out = r.NewPoly()
	for i := range values{
		out[i] = values[i]
	}
	r.NTT(out, out)
	return
}
