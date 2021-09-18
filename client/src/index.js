import Stage from 'stage-js/platform/web';
Stage(function (stage) {
  stage.viewbox(200, 200).on("click", function () {
    document.addEventListener("keydown", (event) => {
      console.log(`${Math.random()} key=${event.key}, code=${event.code}`);
      draw(Math.random());
    });
  });

  var width = 30,
    height = 30;
  var r1 = 10,
    r2 = 10,
    p = 3;

  var image = Stage.image().appendTo(stage).pin("align", 0.5);

  draw();

  function draw(angle=1) {
    image.image(
      Stage.canvas(function (ctx) {
        p = 3;

        this.size(width, height, 4);

        ctx.scale(4, 4);

        // draw star
        ctx.translate(width / 2, height / 2);
        ctx.beginPath();
        ctx.rotate(angle* Math.PI / p);
        ctx.moveTo(0, 0 - r1);
        for (var i = 0; i < p; i++) {
          ctx.rotate(Math.PI / p);
          ctx.lineTo(0, 0 - r2);
          ctx.rotate(Math.PI / p);
          ctx.lineTo(0, 0 - r1);
        }
        // fill & stroke
        ctx.fillStyle = "#eee";
        ctx.fill();
        ctx.lineWidth = 1;
        ctx.strokeStyle = "black";
        ctx.stroke();
      })
    );
  }
});
