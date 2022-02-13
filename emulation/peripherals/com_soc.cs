//
// Copyright (c) 2010-2021 Antmicro
//
// This file is licensed under the MIT License.
// Full license text is available in 'licenses/MIT.txt'.
//
using Antmicro.Renode.Core;
using Antmicro.Renode.Core.Structure;
using Antmicro.Renode.Core.Structure.Registers;
using Antmicro.Renode.Logging;
using Antmicro.Renode.Time;
using Antmicro.Renode.Peripherals.Bus;
using Antmicro.Renode.Peripherals.Timers;

namespace Antmicro.Renode.Peripherals.SPI
{
    public class BetrustedSocSpi : NullRegistrationPointPeripheralContainer<ISPIPeripheral>, IDoubleWordPeripheral, IProvidesRegisterCollection<DoubleWordRegisterCollection>, IKnownSize
    {
        public BetrustedSocSpi(Machine machine) : base(machine)
        {
            IRQ = new GPIO();
            // HOLD = new GPIO();
            RegistersCollection = new DoubleWordRegisterCollection(this);

            Registers.ComTx.Define32(this)
                .WithValueField(0, 16, FieldMode.Write, writeCallback: (_, val) =>
                {
                    // Skip transaction if HOLD is set
                    // if (AutoHold.Value && (HOLD && HOLD.IsSet)
                    // {
                    //     return;
                    // }
                    Rx.Value = (uint)RegisteredPeripheral.Transmit((byte)val);
                    Rx.Value |= ((uint)RegisteredPeripheral.Transmit((byte)(val >> 8))) << 8;
                    if (IntEna.Value)
                    {
                        SpiIntStatus.Value = true;
                        UpdateInterrupts();
                        SpiIntStatus.Value = false;
                    }
                }, name: "TX")
            ;

            Registers.ComRx.Define32(this)
                .WithValueField(0, 16, out Rx, FieldMode.Read, name: "RX")
            ;

            Registers.ComControl.Define32(this)
                .WithFlag(0, out IntEna, FieldMode.Read | FieldMode.Write, name: "IntEna")
                .WithFlag(1, out AutoHold, FieldMode.Read | FieldMode.Write, name: "AutoHold")
            ;
            Registers.ComStatus.Define32(this)
                .WithFlag(0, out Tip, FieldMode.Read, name: "IntEna")
                .WithFlag(1, out Hold, FieldMode.Read, name: "AutoHold")
            ;

            Registers.ComEvStatus.Define32(this)
                .WithFlag(0, out SpiIntStatus, FieldMode.Read, name: "SpiInt")
                .WithFlag(1, out SpiHoldStatus, FieldMode.Read, name: "SpiHold")
            ;

            Registers.ComEvPending.Define32(this)
                .WithFlag(0, out SpiIntPending, FieldMode.Read | FieldMode.WriteOneToClear, name: "SpiInt", changeCallback: (_, __) => UpdateInterrupts())
                .WithFlag(1, out SpiHoldPending, FieldMode.Read | FieldMode.WriteOneToClear, name: "SpiHold", changeCallback: (_, __) => UpdateInterrupts())
            ;

            Registers.ComEvEnable.Define32(this)
                .WithFlag(0, out SpiIntEnable, name: "SpiInt", changeCallback: (_, __) => UpdateInterrupts())
                .WithFlag(1, out SpiHoldEnable, name: "SpiHold", changeCallback: (_, __) => UpdateInterrupts())
            ;

            Reset();
        }

        private void UpdateInterrupts()
        {
            if (SpiIntStatus.Value)
            {
                SpiIntPending.Value = true;
            }
            if (SpiHoldStatus.Value)
            {
                SpiHoldPending.Value = true;
            }
            IRQ.Set((SpiIntPending.Value && SpiIntEnable.Value)
                 || (SpiHoldPending.Value && SpiHoldEnable.Value));
        }

        public void WriteDoubleWord(long address, uint value)
        {
            RegistersCollection.Write(address, value);
        }

        public uint ReadDoubleWord(long offset)
        {
            return RegistersCollection.Read(offset);
        }

        public override void Reset()
        {
            // IntEna.Value = false;
            // AutoHold.Value = false;
            // Tip.Value = false;
            // Rx.Value = 0;

            SpiIntStatus.Value = false;
            SpiHoldStatus.Value = false;
            SpiIntPending.Value = false;
            SpiHoldPending.Value = false;
            SpiIntEnable.Value = false;
            SpiHoldEnable.Value = false;
        }


        public long Size { get { return 0x100; } }

        public GPIO IRQ { get; private set; }
        // public GPIO HOLD;

        private IFlagRegisterField IntEna;
        private IFlagRegisterField AutoHold;
        private IFlagRegisterField Tip;
        private IFlagRegisterField Hold;
        private IValueRegisterField Rx;

        // Interrupts
        private IFlagRegisterField SpiIntStatus;
        private IFlagRegisterField SpiHoldStatus;
        private IFlagRegisterField SpiIntPending;
        private IFlagRegisterField SpiHoldPending;
        private IFlagRegisterField SpiIntEnable;
        private IFlagRegisterField SpiHoldEnable;

        public DoubleWordRegisterCollection RegistersCollection { get; private set; }

        private enum Registers
        {
            ComTx = 0x00,

            ComRx = 0x04,
            ComControl = 0x08,
            ComStatus = 0x0c,
            ComEvStatus = 0x10,
            ComEvPending = 0x14,
            ComEvEnable = 0x18,
        }
    }
}
