using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;
using MAIDOS.IME.AI;

namespace MAIDOS.IME.AI.Tests
{
    public class SmartPredictTests
    {
        [Fact]
        public void SmartPredict_Constructor_Test()
        {
            var predict = new SmartPredict();
            Assert.NotNull(predict);
        }

        [Fact]
        public async Task SmartPredict_AutoCorrect_Test()
        {
            var predict = new SmartPredict();
            var result = await predict.AutoCorrectAsync("test");
            Assert.NotNull(result);
            Assert.Equal("test", result);
        }
    }

    public class VoiceInputTests
    {
        [Fact]
        public void VoiceInput_Constructor_Test()
        {
            var voiceInput = new VoiceInput();
            Assert.NotNull(voiceInput);
        }
    }

    public class HandwritingInputTests
    {
        [Fact]
        public void HandwritingInput_Constructor_Test()
        {
            var handwritingInput = new HandwritingInput();
            Assert.NotNull(handwritingInput);
        }
    }
}