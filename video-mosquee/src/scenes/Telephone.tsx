import React from 'react';
import {AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig} from 'remotion';
import {Kicker, MaskLine, Rise, SceneExit} from '../components/Common';
import {C, pop, progress, sans, serif} from '../theme';

export const TELEPHONE_DURATION = 300;

const CX = 289;
const CY = 480;
const R = 168;

const T = {
	circle: 8,
	phone: 24,
	ring: [44, 92] as const,
	strike: [96, 114] as const,
};

const PhoneIcon: React.FC = () => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const s = pop(frame, fps, T.phone);

	// vibration de la sonnerie (s'arrête quand le trait rouge arrive)
	const ringing = frame >= T.ring[0] && frame < T.strike[0];
	const vib = ringing ? Math.sin(frame * 2.2) * 7 * (Math.floor((frame - T.ring[0]) / 12) % 2 === 0 ? 1 : 0.15) : 0;
	const strike = progress(frame, ...T.strike);
	const waves = ringing ? ((frame - T.ring[0]) % 18) / 18 : 0;

	return (
		<div
			style={{
				position: 'absolute',
				left: CX - 110,
				top: CY - 130,
				width: 220,
				height: 260,
				transform: `scale(${s})`,
			}}
		>
			<svg width={220} height={260} viewBox="0 0 220 260" style={{overflow: 'visible'}}>
				{/* ondes de sonnerie */}
				{ringing
					? [-1, 1].map((side) => (
							<g key={side} opacity={(1 - waves) * 0.9}>
								<path
									d={
										side < 0
											? `M ${22 - waves * 14} 100 Q ${8 - waves * 18} 130 ${22 - waves * 14} 160`
											: `M ${198 + waves * 14} 100 Q ${212 + waves * 18} 130 ${198 + waves * 14} 160`
									}
									fill="none"
									stroke={C.gold}
									strokeWidth={7}
									strokeLinecap="round"
								/>
							</g>
						))
					: null}
				<g transform={`rotate(${vib} 110 130)`}>
					<rect x={46} y={22} width={128} height={216} rx={30} fill="none" stroke={C.green} strokeWidth={17} />
					<rect x={92} y={194} width={36} height={12} rx={6} fill={C.green} />
				</g>
				<line
					x1={30}
					y1={232}
					x2={190}
					y2={32}
					stroke={C.red}
					strokeWidth={15}
					strokeLinecap="butt"
					pathLength={1}
					strokeDasharray={1}
					strokeDashoffset={1 - strike}
				/>
			</svg>
		</div>
	);
};

export const Telephone: React.FC = () => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();

	const c = pop(frame, fps, T.circle);
	const draw = progress(frame, T.circle, T.circle + 30);
	const breathe = frame > T.strike[1] ? 1 + Math.sin((frame - T.strike[1]) / 14) * 0.015 : 1;
	const pulse = interpolate(frame, [T.strike[1], T.strike[1] + 30], [0, 1], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
	});

	const title: React.CSSProperties = {
		fontFamily: serif,
		fontWeight: 700,
		fontSize: 112,
		lineHeight: 1.08,
		color: C.green,
		whiteSpace: 'nowrap',
	};

	return (
		<SceneExit duration={TELEPHONE_DURATION}>
			<AbsoluteFill>
				{/* onde après le trait rouge */}
				<div
					style={{
						position: 'absolute',
						left: CX - R,
						top: CY - R,
						width: R * 2,
						height: R * 2,
						borderRadius: '50%',
						border: `5px solid ${C.gold}`,
						opacity: pulse > 0 && pulse < 1 ? (1 - pulse) * 0.7 : 0,
						transform: `scale(${1 + pulse * 0.35})`,
					}}
				/>
				{/* cercle */}
				<div
					style={{
						position: 'absolute',
						left: CX - R,
						top: CY - R,
						width: R * 2,
						height: R * 2,
						transform: `scale(${interpolate(c, [0, 1], [0.5, 1]) * breathe})`,
						opacity: Math.min(1, c * 2),
					}}
				>
					<svg width={R * 2} height={R * 2} viewBox={`0 0 ${R * 2} ${R * 2}`}>
						<circle cx={R} cy={R} r={R - 4} fill={C.white} />
						<circle
							cx={R}
							cy={R}
							r={R - 4}
							fill="none"
							stroke={C.gold}
							strokeWidth={6}
							pathLength={1}
							strokeDasharray={1}
							strokeDashoffset={1 - draw}
							transform={`rotate(-90 ${R} ${R})`}
						/>
					</svg>
				</div>
				<PhoneIcon />

				<div style={{position: 'absolute', left: 551, top: 236}}>
					<Kicker text="RAPPEL" delay={10} />
				</div>
				<div style={{position: 'absolute', left: 549, top: 314}}>
					<MaskLine delay={18}>
						<div style={title}>Téléphone en</div>
					</MaskLine>
					<MaskLine delay={26} style={{marginTop: -8}}>
						<div style={title}>silencieux</div>
					</MaskLine>
				</div>
				<div style={{position: 'absolute', left: 551, top: 600, width: 1200}}>
					<Rise delay={44} distance={30}>
						<div style={{fontFamily: sans, fontWeight: 400, fontSize: 52, lineHeight: 1.36, color: C.text}}>
							Merci de couper la sonnerie avant d&apos;entrer dans la salle de prière.
						</div>
					</Rise>
				</div>
			</AbsoluteFill>
		</SceneExit>
	);
};
